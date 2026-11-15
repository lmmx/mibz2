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

The latest record: [`1c961aa`](<https://github.com/lmmx/mibz2/commit/1c961aa29d60fece4d9f003fec22ce125e547e16>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43728461973>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.496 s | 423 | 1.9 | 2.98× (2.93–3.02) | 6.66× (6.54–6.78) | 7.09× (6.95–7.23) | 7.98× (7.81–8.14) | 7.11× (6.96–7.26) | 7.63× (7.48–7.77) | 7.33× (7.17–7.49) | 2.29× (2.24–2.34) | 2.14× (2.09–2.19) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.281 s | 746 | 2.1 | 2.92× (2.77–3.08) | 6.13× (5.78–6.49) | 7.42× (7.04–7.81) | 7.31× (6.90–7.75) | 6.55× (6.20–6.92) | 12.97× (12.28–13.69) | 12.46× (11.77–13.19) | 3.89× (3.67–4.13) | 3.64× (3.44–3.85) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.223 s | 939 | 3.1 | 3.25× (3.22–3.28) | 5.66× (5.60–5.73) | 6.41× (6.30–6.51) | 6.41× (6.33–6.48) | 6.31× (6.23–6.40) | 16.92× (16.74–17.10) | 16.26× (16.04–16.49) | 5.08× (5.00–5.16) | 4.75× (4.68–4.81) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,047 | 0.8 | 0.58× (0.56–0.61) | 3.13× (3.01–3.25) | 4.40× (4.15–4.67) | 3.76× (3.62–3.90) | 6.21× (6.01–6.41) | 2.82× (2.73–2.91) | 2.79× (2.68–2.91) | 1.10× (1.05–1.15) | 1.06× (1.02–1.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 1,020 | 0.9 | 0.55× (0.53–0.56) | 3.13× (3.07–3.19) | 5.71× (5.60–5.82) | 3.78× (3.71–3.84) | 6.17× (6.07–6.28) | 2.81× (2.78–2.84) | 2.79× (2.73–2.84) | 1.10× (1.07–1.12) | 1.05× (1.03–1.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,028 | 0.8 | 0.56× (0.54–0.58) | 3.17× (3.09–3.26) | 5.77× (5.62–5.93) | 3.82× (3.68–3.96) | 6.27× (6.07–6.47) | 2.84× (2.77–2.91) | 2.82× (2.72–2.91) | 1.11× (1.06–1.15) | 1.06× (1.04–1.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 569 | 1.5 | 2.87× (2.81–2.93) | 3.45× (3.38–3.52) | 3.94× (3.82–4.06) | 4.55× (4.45–4.64) | 4.92× (4.83–5.01) | 3.68× (3.57–3.80) | 3.02× (2.95–3.08) | 2.11× (2.07–2.16) | 2.22× (2.13–2.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 893 | 1.8 | 2.35× (2.18–2.52) | 3.10× (2.88–3.33) | 4.13× (3.78–4.52) | 3.93× (3.72–4.14) | 5.33× (4.93–5.75) | 5.62× (5.25–6.02) | 4.60× (4.31–4.91) | 3.22× (3.01–3.44) | 3.38× (3.19–3.59) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 916 | 3.1 | 2.21× (2.14–2.30) | 2.09× (1.99–2.20) | 3.17× (2.91–3.45) | 2.66× (2.48–2.86) | 4.57× (4.36–4.80) | 5.92× (5.64–6.21) | 4.85× (4.66–5.06) | 3.39× (3.26–3.53) | 3.56× (3.33–3.82) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.136 s | 59 | 13.8 | 1.34× (1.32–1.35) | 2.70× (2.66–2.74) | 2.43× (2.39–2.48) | 2.52× (2.48–2.56) | 2.77× (2.73–2.80) | 3.12× (3.08–3.17) | 3.20× (3.18–3.23) | 1.26× (1.24–1.28) | 1.30× (1.27–1.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 98 | 15.4 | 1.32× (1.25–1.39) | 2.63× (2.57–2.69) | 2.51× (2.43–2.59) | 3.22× (3.13–3.31) | 2.92× (2.86–2.98) | 5.19× (5.08–5.29) | 5.32× (5.21–5.43) | 2.09× (2.04–2.14) | 2.16× (2.11–2.21) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 119 | 20.8 | 1.37× (1.33–1.41) | 2.46× (2.37–2.55) | 2.36× (2.24–2.49) | 2.90× (2.78–3.03) | 2.90× (2.83–2.97) | 6.18× (6.02–6.35) | 6.34× (6.15–6.53) | 2.49× (2.41–2.57) | 2.58× (2.48–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.152 s | 63 | 12.8 | 1.39× (1.37–1.40) | 3.10× (3.05–3.15) | 2.57× (2.54–2.60) | 2.69× (2.65–2.73) | 3.01× (2.97–3.04) | 3.81× (3.76–3.86) | 4.05× (3.99–4.10) | 1.47× (1.45–1.50) | 1.54× (1.51–1.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 106 | 14.3 | 1.30× (1.26–1.33) | 2.91× (2.83–2.99) | 2.57× (2.50–2.63) | 3.57× (3.50–3.64) | 3.07× (3.00–3.14) | 6.40× (6.26–6.55) | 6.80× (6.66–6.95) | 2.48× (2.41–2.55) | 2.59× (2.52–2.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.074 s | 130 | 19.9 | 1.46× (1.43–1.50) | 2.78× (2.69–2.87) | 2.43× (2.35–2.51) | 3.30× (3.19–3.42) | 3.00× (2.95–3.06) | 7.72× (7.54–7.90) | 8.21× (8.03–8.39) | 2.99× (2.92–3.06) | 3.12× (3.04–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 775 | 1.2 | 0.61× (0.60–0.62) | 3.15× (3.08–3.22) | 3.92× (3.83–4.01) | 3.18× (3.11–3.25) | 6.69× (6.58–6.80) | 2.39× (2.34–2.44) | 2.37× (2.33–2.42) | 1.02× (1.00–1.03) | 1.21× (0.96–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 752 | 1.2 | 0.54× (0.53–0.55) | 2.70× (2.45–2.98) | 5.25× (5.03–5.49) | 3.19× (3.13–3.25) | 6.56× (6.40–6.72) | 2.35× (2.29–2.42) | 2.34× (2.29–2.38) | 1.00× (0.98–1.02) | 1.19× (0.95–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 746 | 1.2 | 0.54× (0.52–0.55) | 2.95× (2.77–3.14) | 5.03× (4.70–5.38) | 3.17× (2.91–3.45) | 6.37× (6.10–6.66) | 2.29× (2.21–2.37) | 2.27× (2.19–2.36) | 0.97× (0.94–1.01) | 1.16× (0.93–1.46) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.069 s | 183 | 4.5 | 1.76× (1.74–1.78) | 3.93× (3.88–3.98) | 4.30× (4.23–4.38) | 4.54× (4.45–4.64) | 4.67× (4.60–4.74) | 4.93× (4.88–4.98) | 4.84× (4.78–4.89) | 1.48× (1.45–1.50) | 1.48× (1.46–1.51) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.045 s | 278 | 5.3 | 1.47× (1.39–1.55) | 3.24× (3.03–3.46) | 3.96× (3.77–4.16) | 4.42× (4.22–4.64) | 4.25× (4.06–4.45) | 7.28× (7.01–7.57) | 7.15× (6.88–7.43) | 2.18× (2.09–2.28) | 2.19× (2.09–2.29) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.040 s | 319 | 7.8 | 1.56× (1.49–1.63) | 2.67× (2.56–2.78) | 3.35× (3.28–3.42) | 3.90× (3.75–4.06) | 3.97× (3.91–4.03) | 8.46× (8.28–8.64) | 8.30× (8.12–8.49) | 2.53× (2.48–2.58) | 2.54× (2.50–2.58) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.97× (2.93–3.02) | 6.64× (6.52–6.76) | 7.06× (6.92–7.20) | 7.95× (7.79–8.12) | 7.09× (6.94–7.24) | 7.60× (7.46–7.75) | 7.31× (7.15–7.46) | 2.29× (2.24–2.33) | 2.13× (2.08–2.17) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.87× (2.85–2.90) | 6.48× (6.38–6.58) | 7.10× (6.98–7.21) | 7.86× (7.78–7.94) | 6.74× (6.69–6.80) | 7.02× (6.95–7.10) | 6.75× (6.69–6.81) | 2.11× (2.09–2.14) | 1.97× (1.95–1.98) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.13× (3.07–3.19) | 6.04× (5.95–6.14) | 6.15× (6.05–6.25) | 6.84× (6.74–6.93) | 6.12× (6.03–6.21) | 4.61× (4.54–4.67) | 4.43× (4.36–4.50) | 1.38× (1.37–1.40) | 1.29× (1.27–1.31) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.64) | 2.86× (2.76–2.96) | 3.91× (3.78–4.05) | 3.55× (3.42–3.68) | 5.81× (5.65–5.99) | 2.68× (2.60–2.76) | 2.65× (2.55–2.76) | 1.09× (1.04–1.14) | 0.90× (0.86–0.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.61) | 2.86× (2.79–2.93) | 5.11× (5.05–5.17) | 3.54× (3.49–3.59) | 5.71× (5.63–5.80) | 2.63× (2.60–2.67) | 2.61× (2.57–2.65) | 1.07× (1.05–1.09) | 0.88× (0.86–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.57–0.61) | 3.34× (3.22–3.46) | 5.20× (5.04–5.36) | 3.63× (3.50–3.76) | 5.86× (5.67–6.05) | 2.70× (2.64–2.76) | 2.68× (2.59–2.77) | 1.10× (1.06–1.14) | 0.90× (0.88–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.79× (2.73–2.84) | 3.19× (3.14–3.24) | 3.70× (3.63–3.77) | 4.40× (4.31–4.49) | 4.74× (4.66–4.83) | 3.57× (3.46–3.68) | 2.93× (2.87–2.99) | 2.06× (2.02–2.10) | 1.99× (1.96–2.03) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.45× (2.36–2.54) | 2.84× (2.72–2.96) | 3.32× (3.20–3.44) | 3.71× (3.57–3.84) | 4.07× (3.91–4.22) | 2.96× (2.87–3.05) | 2.43× (2.36–2.51) | 1.71× (1.66–1.77) | 1.65× (1.59–1.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.30× (2.21–2.39) | 1.99× (1.91–2.08) | 2.41× (2.34–2.49) | 2.39× (2.33–2.46) | 2.51× (2.44–2.57) | 1.68× (1.65–1.72) | 1.38× (1.35–1.42) | 0.97× (0.94–1.00) | 0.94× (0.91–0.97) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.35) | 2.65× (2.61–2.68) | 2.37× (2.33–2.41) | 2.50× (2.46–2.54) | 2.75× (2.71–2.78) | 3.10× (3.06–3.14) | 3.18× (3.15–3.20) | 1.25× (1.23–1.28) | 1.26× (1.24–1.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.28) | 2.57× (2.53–2.60) | 2.26× (2.22–2.29) | 2.94× (2.90–2.99) | 2.57× (2.53–2.60) | 2.77× (2.72–2.81) | 2.84× (2.79–2.89) | 1.12× (1.10–1.14) | 1.12× (1.10–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.49–1.57) | 2.63× (2.59–2.68) | 2.22× (2.15–2.28) | 2.84× (2.78–2.90) | 2.57× (2.53–2.61) | 2.06× (2.02–2.10) | 2.11× (2.07–2.15) | 0.83× (0.81–0.85) | 0.83× (0.82–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.38× (1.37–1.40) | 3.04× (2.99–3.09) | 2.51× (2.47–2.55) | 2.67× (2.63–2.71) | 2.98× (2.95–3.02) | 3.77× (3.72–3.82) | 4.01× (3.96–4.06) | 1.47× (1.44–1.50) | 1.49× (1.47–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.29× (1.25–1.32) | 2.88× (2.80–2.96) | 2.37× (2.32–2.43) | 3.26× (3.15–3.37) | 2.76× (2.69–2.84) | 3.35× (3.27–3.44) | 3.56× (3.48–3.65) | 1.30× (1.27–1.34) | 1.33× (1.29–1.37) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.54–1.66) | 3.02× (2.94–3.09) | 2.22× (2.17–2.27) | 3.10× (3.01–3.19) | 2.73× (2.66–2.79) | 2.46× (2.40–2.51) | 2.61× (2.55–2.67) | 0.95× (0.93–0.98) | 0.97× (0.94–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.64–0.67) | 2.40× (2.35–2.44) | 3.41× (3.36–3.45) | 2.94× (2.88–3.01) | 6.06× (5.95–6.17) | 2.24× (2.19–2.29) | 2.23× (2.19–2.28) | 1.02× (1.00–1.04) | 0.82× (0.81–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.31× (2.26–2.36) | 4.24× (4.14–4.34) | 2.96× (2.91–3.01) | 5.87× (5.74–6.01) | 2.17× (2.11–2.23) | 2.16× (2.11–2.21) | 0.99× (0.97–1.00) | 0.80× (0.78–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.60) | 2.65× (2.57–2.74) | 4.14× (3.99–4.29) | 3.01× (2.78–3.27) | 5.84× (5.60–6.09) | 2.16× (2.09–2.24) | 2.15× (2.08–2.23) | 0.98× (0.95–1.02) | 0.79× (0.77–0.82) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.74× (1.72–1.77) | 3.77× (3.73–3.80) | 4.16× (4.10–4.23) | 4.46× (4.36–4.56) | 4.58× (4.51–4.65) | 4.83× (4.78–4.88) | 4.74× (4.69–4.80) | 1.46× (1.44–1.48) | 1.38× (1.36–1.39) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.54× (1.47–1.62) | 3.27× (3.12–3.44) | 3.77× (3.59–3.95) | 4.14× (3.95–4.35) | 3.86× (3.67–4.05) | 3.95× (3.78–4.13) | 3.88× (3.71–4.05) | 1.20× (1.14–1.26) | 1.12× (1.08–1.18) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.83× (1.73–1.93) | 3.05× (2.96–3.15) | 3.45× (3.38–3.52) | 3.70× (3.63–3.76) | 3.54× (3.46–3.62) | 2.75× (2.69–2.82) | 2.70× (2.63–2.78) | 0.83× (0.82–0.85) | 0.78× (0.77–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.05× (11.03–11.07) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.30–0.33) | 0.43× (0.36–0.50) | 0.65× (0.62–0.68) | 0.54× (0.52–0.57) | 5.19× (4.95–5.44) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 95 MB | 0.32× (0.31–0.34) | 0.49× (0.40–0.59) | 0.58× (0.55–0.61) | 0.51× (0.48–0.54) | 3.38× (3.21–3.56) | 0.08× (0.07–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.45) | 1.70× (1.69–1.72) | 0.66× (0.65–0.66) | 8.04× (7.99–8.10) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.65× (0.61–0.70) | 1.61× (1.57–1.64) | 0.80× (0.78–0.82) | 5.83× (5.70–5.98) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.66) | 0.71× (0.69–0.74) | 1.16× (1.15–1.18) | 0.71× (0.69–0.73) | 3.47× (3.42–3.53) | 0.27× (0.27–0.27) | 0.29× (0.28–0.29) | 0.31× (0.31–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.22× (5.21–5.23) | 0.33× (0.32–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.38× (0.35–0.42) | 0.88× (0.87–0.90) | 0.70× (0.68–0.73) | 3.05× (2.99–3.11) | 0.19× (0.19–0.19) | 0.20× (0.20–0.21) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.80× (0.77–0.82) | 2.50× (2.45–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.83) | 5.21× (5.20–5.23) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.48× (0.46–0.49) | 0.36× (0.33–0.38) | 0.83× (0.81–0.85) | 0.69× (0.67–0.71) | 2.86× (2.80–2.93) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.47× (0.44–0.51) | 0.81× (0.80–0.83) | 0.83× (0.80–0.86) | 2.35× (2.32–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.36–5.40) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.30× (5.28–5.32) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.77) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.65–0.67) | 6.39× (6.36–6.42) | 0.39× (0.39–0.39) | 0.43× (0.42–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.32) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.39× (0.35–0.43) | 1.02× (0.99–1.05) | 0.66× (0.64–0.68) | 3.23× (3.17–3.29) | 0.20× (0.19–0.20) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.15–0.16) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 58 MB | 0.48× (0.46–0.50) | 0.40× (0.37–0.44) | 0.88× (0.86–0.90) | 0.71× (0.68–0.74) | 2.26× (2.21–2.32) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 423 | 2.37 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 746 | 2.58 | 1.08× (1.06–1.11) | 85% | 1.06× (1.04–1.07) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 939 | 3.89 | 1.65× (1.62–1.69) | 55% | 1.50× (1.48–1.52) | 1.74× (1.70–1.78) |

**Best:** mibz2 at 4 threads (939 MB/s); lbzip2 at 4 threads (166 MB/s); pbz2 at 4 threads (288 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.18, lbzip2 1t 0.04, lbzip2 2t 0.23, lbzip2 4t 0.32, pbz2 1t 0.00, pbz2 2t 0.35, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.61 | 0.02 | 0.08 | 0.223 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.29 | 11.7% |
| MTF + RLE2 | 0.41 | 16.7% |
| T-vector build | 0.39 | 15.9% |
| walk (the one each build uses) | 1.27 | 51.8% |
| of which: segment assembly | 0.07 | 2.7% |
| RLE1 | 0.07 | 2.7% |
| CRC | 0.03 | 1.4% |
| **total** | 2.46 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.463 | 1.204 | (in its total) | 2.60× (2.57–2.62) |  |
| walk | 1.492 | 2.861 | (in its total) | 1.92× (1.91–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.083 | 0.076 | (in its total) | 0.92× (0.91–0.92) |  |
| inverse BWT, total | 2.037 | 4.141 | 2.186 | 2.03× (2.02–2.04) | 1.07× (1.06–1.09) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h11m ± 8m20s |
| 4 threads | 32m22s ± 4m34s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m33s ± 3m13s |
| the scanner alone (its serial part) | 13m32s ± 30.2s |
| the workers alone, over 4 | 22m33s ± 3m13s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 2.96 ns a byte of output at 4 threads (CPU, summed over them), 1.91 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,508,059 | 12,616,482 | \`8d43d83ee25ebeaf…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
