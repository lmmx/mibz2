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

The latest record: [`07a1b1d`](<https://github.com/lmmx/mibz2/commit/07a1b1d09f8ac872d90251cc6987479496493381>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46143817670>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.316 s | 664 | 6.3 | 2.93× (2.86–3.01) | 7.36× (7.18–7.55) | 7.68× (7.42–7.95) | 8.76× (8.46–9.06) | 7.82× (7.52–8.13) | 7.92× (7.67–8.19) | 7.50× (7.19–7.82) | 2.32× (2.23–2.42) | 2.65× (2.56–2.75) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.194 s | 1,084 | 7.0 | 3.14× (2.94–3.36) | 6.57× (6.34–6.80) | 8.14× (7.86–8.43) | 7.95× (7.73–8.19) | 7.06× (6.74–7.40) | 13.00× (12.25–13.79) | 12.30× (11.70–12.93) | 3.80× (3.67–3.95) | 4.35× (4.17–4.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.158 s | 1,325 | 10.5 | 3.11× (2.96–3.26) | 4.72× (4.50–4.94) | 6.28× (6.00–6.57) | 5.73× (5.51–5.97) | 5.68× (5.43–5.95) | 15.61× (14.76–16.51) | 14.77× (14.04–15.54) | 4.57× (4.34–4.81) | 5.22× (4.94–5.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,205 | 3.9 | 0.54× (0.50–0.58) | 3.50× (3.20–3.83) | 4.33× (4.04–4.64) | 3.55× (3.34–3.77) | 6.34× (5.86–6.86) | 2.57× (2.40–2.75) | 2.66× (2.45–2.88) | 0.98× (0.91–1.05) | 1.86× (1.74–1.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,153 | 4.2 | 0.47× (0.44–0.50) | 2.97× (2.81–3.13) | 5.64× (5.11–6.21) | 3.52× (3.31–3.75) | 5.96× (5.53–6.43) | 2.44× (2.28–2.62) | 2.53× (2.36–2.71) | 0.93× (0.88–0.99) | 1.76× (1.67–1.86) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,163 | 4.1 | 0.45× (0.42–0.48) | 2.84× (2.57–3.14) | 5.49× (5.03–5.99) | 3.29× (3.05–3.55) | 5.89× (5.45–6.38) | 2.39× (2.22–2.57) | 2.47× (2.33–2.63) | 0.91× (0.83–1.00) | 1.73× (1.59–1.87) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 973 | 4.7 | 2.98× (2.87–3.10) | 3.68× (3.43–3.94) | 4.41× (4.13–4.71) | 4.80× (4.51–5.12) | 5.64× (5.08–6.26) | 3.20× (3.02–3.40) | 2.76× (2.56–2.98) | 2.30× (2.22–2.39) | 3.20× (2.98–3.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,320 | 6.0 | 2.29× (2.01–2.60) | 3.07× (2.76–3.41) | 4.49× (4.07–4.96) | 4.02× (3.72–4.35) | 6.11× (5.53–6.76) | 4.26× (3.98–4.56) | 3.67× (3.34–4.04) | 3.07× (2.85–3.30) | 4.26× (3.80–4.78) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,133 | 11.6 | 1.86× (1.69–2.06) | 1.98× (1.72–2.28) | 3.32× (2.68–4.12) | 2.35× (2.12–2.61) | 4.64× (4.12–5.22) | 3.75× (3.45–4.07) | 3.23× (2.88–3.63) | 2.70× (2.46–2.96) | 3.75× (3.31–4.24) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.085 s | 95 | 45.1 | 1.48× (1.42–1.55) | 3.33× (3.18–3.49) | 3.02× (2.87–3.17) | 2.95× (2.84–3.06) | 3.44× (3.15–3.75) | 3.78× (3.62–3.96) | 3.70× (3.56–3.84) | 1.42× (1.37–1.47) | 1.64× (1.58–1.71) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.051 s | 158 | 50.3 | 1.41× (1.31–1.52) | 3.16× (2.98–3.35) | 3.06× (2.87–3.25) | 3.85× (3.62–4.08) | 3.61× (3.43–3.81) | 6.19× (5.73–6.67) | 6.04× (5.70–6.40) | 2.32× (2.15–2.50) | 2.69× (2.57–2.81) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.042 s | 191 | 65.2 | 1.43× (1.34–1.52) | 2.69× (2.52–2.88) | 2.74× (2.58–2.91) | 3.16× (2.89–3.47) | 3.53× (3.31–3.76) | 7.39× (6.66–8.19) | 7.22× (6.61–7.88) | 2.77× (2.49–3.08) | 3.21× (2.96–3.48) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.088 s | 109 | 39.0 | 1.67× (1.51–1.85) | 3.82× (3.66–3.99) | 3.17× (3.09–3.25) | 3.32× (3.10–3.55) | 3.67× (3.59–3.77) | 4.75× (4.61–4.90) | 5.10× (4.74–5.50) | 1.69× (1.59–1.79) | 1.91× (1.82–2.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.057 s | 168 | 46.3 | 1.41× (1.34–1.48) | 3.47× (3.25–3.72) | 3.02× (2.75–3.32) | 4.08× (3.82–4.37) | 3.54× (3.32–3.76) | 7.54× (7.16–7.94) | 8.10× (7.48–8.77) | 2.68× (2.47–2.89) | 3.03× (2.86–3.21) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.046 s | 207 | 62.6 | 1.51× (1.43–1.60) | 2.95× (2.75–3.18) | 2.83× (2.65–3.03) | 3.66× (3.46–3.88) | 3.64× (3.35–3.96) | 9.12× (8.62–9.64) | 9.79× (8.96–10.71) | 3.23× (3.00–3.48) | 3.66× (3.43–3.92) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 930 | 5.3 | 0.59× (0.56–0.62) | 3.03× (2.66–3.46) | 4.35× (4.05–4.68) | 3.15× (2.96–3.36) | 7.18× (6.96–7.41) | 2.30× (2.16–2.44) | 2.28× (2.21–2.35) | 0.98× (0.91–1.04) | 1.95× (1.89–2.01) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 905 | 5.6 | 0.47× (0.42–0.53) | 2.75× (2.59–2.93) | 4.93× (4.33–5.61) | 2.86× (2.60–3.16) | 6.50× (6.03–7.01) | 2.13× (1.87–2.41) | 2.11× (1.94–2.29) | 0.90× (0.79–1.03) | 1.80× (1.67–1.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 901 | 5.5 | 0.48× (0.43–0.54) | 2.67× (2.46–2.89) | 4.74× (4.21–5.34) | 2.91× (2.61–3.26) | 6.61× (6.05–7.22) | 2.12× (1.89–2.36) | 2.10× (1.93–2.28) | 0.90× (0.80–1.01) | 1.80× (1.65–1.96) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 0.043 s | 350 | 12.3 | 1.97× (1.88–2.06) | 4.75× (4.24–5.31) | 4.98× (4.43–5.60) | 5.00× (4.64–5.40) | 5.31× (5.07–5.55) | 5.15× (4.91–5.41) | 4.94× (4.67–5.22) | 1.70× (1.59–1.80) | 2.03× (1.84–2.23) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 0.032 s | 478 | 16.6 | 1.44× (1.32–1.56) | 3.35× (3.02–3.72) | 4.17× (3.88–4.47) | 4.51× (4.14–4.91) | 4.47× (4.15–4.82) | 6.85× (6.32–7.42) | 6.56× (6.07–7.10) | 2.26× (2.08–2.44) | 2.70× (2.34–3.11) | yes |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 0.028 s | 534 | 25.2 | 1.48× (1.41–1.55) | 2.70× (2.50–2.92) | 3.79× (3.51–4.09) | 3.89× (3.65–4.15) | 4.38× (4.09–4.70) | 7.95× (7.39–8.54) | 7.61× (7.10–8.16) | 2.62× (2.47–2.77) | 3.13× (2.86–3.42) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.92× (2.85–3.00) | 7.32× (7.14–7.50) | 7.62× (7.36–7.89) | 8.71× (8.42–9.02) | 7.78× (7.48–8.09) | 7.88× (7.63–8.15) | 7.46× (7.16–7.78) | 2.31× (2.22–2.41) | 2.63× (2.53–2.73) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.94× (2.74–3.16) | 6.85× (6.66–7.05) | 7.54× (7.39–7.69) | 8.35× (8.25–8.45) | 7.10× (6.92–7.29) | 6.99× (6.70–7.30) | 6.62× (6.37–6.87) | 2.05× (1.97–2.14) | 2.33× (2.26–2.40) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.08× (2.93–3.24) | 5.26× (4.96–5.58) | 5.98× (5.70–6.29) | 6.45× (6.03–6.89) | 5.57× (5.20–5.97) | 4.56× (4.24–4.90) | 4.31× (4.01–4.64) | 1.34× (1.25–1.43) | 1.52× (1.42–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.54–0.62) | 2.92× (2.73–3.11) | 3.68× (3.44–3.94) | 3.29× (3.12–3.48) | 5.82× (5.38–6.30) | 2.43× (2.27–2.60) | 2.49× (2.31–2.69) | 0.98× (0.92–1.05) | 1.67× (1.58–1.76) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.52× (0.48–0.55) | 2.63× (2.50–2.77) | 4.57× (4.33–4.82) | 3.18× (3.00–3.36) | 5.29× (4.93–5.68) | 2.22× (2.10–2.35) | 2.28× (2.14–2.43) | 0.90× (0.85–0.96) | 1.53× (1.46–1.60) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.48× (0.45–0.51) | 2.67× (2.47–2.88) | 4.49× (4.13–4.89) | 3.01× (2.80–3.23) | 5.25× (4.84–5.70) | 2.20× (2.02–2.38) | 2.25× (2.11–2.41) | 0.89× (0.81–0.98) | 1.51× (1.39–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.81× (2.70–2.93) | 3.26× (3.09–3.44) | 3.92× (3.71–4.15) | 4.49× (4.22–4.77) | 5.25× (4.75–5.81) | 3.02× (2.85–3.20) | 2.61× (2.43–2.81) | 2.19× (2.11–2.27) | 2.82× (2.70–2.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.35× (2.15–2.56) | 2.81× (2.58–3.05) | 3.26× (3.05–3.49) | 3.73× (3.53–3.95) | 4.31× (4.03–4.61) | 2.28× (2.07–2.51) | 1.97× (1.84–2.12) | 1.65× (1.55–1.77) | 2.13× (1.97–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.96× (1.78–2.15) | 1.81× (1.64–1.98) | 2.32× (1.99–2.69) | 2.22× (2.07–2.38) | 2.44× (2.28–2.61) | 1.18× (1.10–1.27) | 1.02× (0.96–1.09) | 0.86× (0.80–0.91) | 1.10× (1.02–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.47× (1.41–1.54) | 3.23× (3.10–3.37) | 2.91× (2.77–3.05) | 2.90× (2.80–3.01) | 3.38× (3.11–3.69) | 3.72× (3.56–3.89) | 3.64× (3.51–3.77) | 1.41× (1.36–1.46) | 1.56× (1.49–1.63) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.31–1.47) | 3.04× (2.88–3.21) | 2.71× (2.57–2.85) | 3.47× (3.31–3.62) | 3.09× (2.96–3.23) | 3.29× (3.10–3.50) | 3.21× (3.08–3.35) | 1.25× (1.18–1.32) | 1.38× (1.33–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.56× (1.47–1.65) | 2.87× (2.72–3.03) | 2.41× (2.31–2.52) | 3.11× (2.90–3.35) | 2.95× (2.82–3.10) | 2.45× (2.25–2.66) | 2.39× (2.22–2.57) | 0.93× (0.85–1.01) | 1.02× (0.95–1.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.66× (1.50–1.83) | 3.72× (3.56–3.89) | 3.09× (3.00–3.17) | 3.27× (3.06–3.50) | 3.62× (3.54–3.71) | 4.68× (4.54–4.83) | 5.03× (4.67–5.41) | 1.68× (1.58–1.77) | 1.83× (1.75–1.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.44× (1.40–1.49) | 3.45× (3.33–3.57) | 2.74× (2.61–2.87) | 3.84× (3.72–3.97) | 3.18× (3.05–3.32) | 4.03× (3.94–4.13) | 4.33× (4.02–4.67) | 1.44× (1.36–1.53) | 1.58× (1.52–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.62–1.72) | 3.19× (2.99–3.41) | 2.36× (2.23–2.51) | 3.36× (3.20–3.53) | 3.18× (3.01–3.37) | 2.94× (2.81–3.07) | 3.16× (2.97–3.36) | 1.05× (1.00–1.11) | 1.15× (1.09–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.64× (0.61–0.67) | 2.53× (2.27–2.83) | 3.49× (3.28–3.72) | 2.83× (2.65–3.01) | 6.28× (6.10–6.47) | 2.10× (1.99–2.23) | 2.09× (2.04–2.14) | 0.98× (0.92–1.04) | 1.45× (1.34–1.56) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.53× (0.48–0.58) | 2.20× (2.03–2.38) | 3.77× (3.44–4.13) | 2.52× (2.30–2.75) | 5.45× (5.09–5.83) | 1.86× (1.65–2.08) | 1.84× (1.70–1.99) | 0.86× (0.76–0.98) | 1.28× (1.12–1.45) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.48–0.58) | 2.33× (2.13–2.53) | 3.65× (3.36–3.96) | 2.67× (2.41–2.95) | 5.72× (5.29–6.19) | 1.92× (1.73–2.12) | 1.90× (1.77–2.04) | 0.89× (0.80–0.98) | 1.32× (1.18–1.47) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 1.93× (1.85–2.02) | 4.45× (3.97–4.98) | 4.76× (4.25–5.33) | 4.85× (4.51–5.22) | 5.14× (4.92–5.37) | 5.00× (4.77–5.24) | 4.78× (4.53–5.05) | 1.68× (1.57–1.79) | 1.90× (1.73–2.09) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 1.53× (1.43–1.64) | 3.34× (3.06–3.64) | 3.68× (3.50–3.87) | 4.23× (3.98–4.50) | 3.96× (3.74–4.19) | 3.74× (3.49–4.02) | 3.59× (3.37–3.81) | 1.26× (1.19–1.32) | 1.43× (1.26–1.61) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 1.63× (1.56–1.69) | 2.70× (2.58–2.83) | 3.12× (2.92–3.33) | 3.46× (3.25–3.68) | 3.31× (3.16–3.47) | 2.49× (2.34–2.64) | 2.38× (2.27–2.49) | 0.83× (0.80–0.87) | 0.95× (0.86–1.04) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.79× (0.78–0.80) | 11.00× (10.98–11.02) | 0.26× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.46× (0.37–0.56) | 0.69× (0.66–0.71) | 0.59× (0.58–0.60) | 5.56× (5.42–5.70) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.35–0.38) | 0.61× (0.49–0.76) | 0.65× (0.62–0.69) | 0.57× (0.55–0.60) | 3.77× (3.60–3.95) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.26× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.19) | 1.95× (1.95–1.96) | 0.92× (0.92–0.92) | 4.83× (4.82–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.95) | 0.92× (0.92–0.93) | 4.84× (4.83–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.67–1.69) | 0.66× (0.65–0.67) | 7.94× (7.90–7.97) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.75) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.68× (0.63–0.73) | 1.60× (1.54–1.66) | 0.81× (0.79–0.83) | 5.79× (5.59–5.99) | 0.47× (0.45–0.48) | 0.49× (0.48–0.51) | 0.54× (0.52–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.67× (0.63–0.71) | 0.72× (0.68–0.77) | 1.20× (1.15–1.26) | 0.73× (0.69–0.78) | 3.49× (3.33–3.66) | 0.27× (0.26–0.29) | 0.29× (0.27–0.30) | 0.31× (0.30–0.33) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.15× (5.14–5.16) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.40× (0.37–0.42) | 0.91× (0.88–0.93) | 0.71× (0.68–0.75) | 3.14× (3.04–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.46) | 0.86× (0.85–0.88) | 0.80× (0.77–0.82) | 2.51× (2.47–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.17–1.18) | 0.84× (0.83–0.84) | 5.22× (5.20–5.23) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.46× (0.44–0.48) | 0.36× (0.34–0.39) | 0.81× (0.78–0.84) | 0.70× (0.68–0.73) | 2.80× (2.69–2.91) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.62) | 0.46× (0.44–0.49) | 0.79× (0.78–0.81) | 0.82× (0.80–0.85) | 2.30× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.29–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.23–0.25) | 1.86× (1.85–1.87) | 0.89× (0.88–0.89) | 6.17× (6.14–6.20) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.23) | 1.85× (1.85–1.86) | 0.89× (0.88–0.90) | 6.15× (6.13–6.17) | 0.40× (0.39–0.40) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 1 | 20 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.38) | 1.38× (1.38–1.38) | 0.66× (0.66–0.66) | 6.35× (6.34–6.36) | 0.38× (0.38–0.39) | 0.41× (0.41–0.41) | 0.63× (0.62–0.63) | 0.30× (0.30–0.30) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 2 | 36 MB | 0.46× (0.45–0.47) | 0.40× (0.36–0.44) | 1.07× (1.04–1.11) | 0.71× (0.69–0.73) | 3.46× (3.39–3.54) | 0.21× (0.21–0.21) | 0.22× (0.22–0.23) | 0.34× (0.34–0.35) | 0.16× (0.16–0.17) |
| `text` (2.27 MB → 15.1 MB, 6.6×) | 4 | 56 MB | 0.50× (0.49–0.51) | 0.44× (0.40–0.49) | 0.90× (0.89–0.92) | 0.73× (0.72–0.75) | 2.31× (2.28–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 664 | 1.51 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,084 | 1.69 | 1.13× (1.09–1.17) | 82% | 1.06× (1.03–1.08) | 1.14× (1.07–1.21) |
| 4: all 4 CPUs, sharing SMT siblings | 1,325 | 2.53 | 1.73× (1.62–1.84) | 49% | 1.24× (1.21–1.27) | 1.83× (1.73–1.92) |

**Best:** mibz2 at 4 threads (1,325 MB/s); lbzip2 at 4 threads (282 MB/s); pbz2 at 4 threads (422 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.11, lbzip2 1t 0.07, lbzip2 2t 0.31, lbzip2 4t 0.43, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.41 | 0.01 | 0.07 | 0.158 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4158 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.71 | 10.4% |
| MTF + RLE2 | 1.44 | 21.0% |
| T-vector build | 1.29 | 18.8% |
| walk (the one each build uses) | 3.00 | 43.8% |
| of which: segment assembly | 0.12 | 1.7% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.25 | 3.6% |
| **total** | 6.84 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.313 | 0.754 | (in its total) | 2.40× (2.39–2.41) |  |
| walk | 0.700 | 1.698 | (in its total) | 2.42× (2.40–2.44) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.033 | 0.034 | (in its total) | 1.02× (0.98–1.06) |  |
| inverse BWT, total | 1.048 | 2.485 | 1.404 | 2.37× (2.36–2.38) | 1.34× (1.33–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m54s ± 5m38s |
| 4 threads | 23m19s ± 3m26s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m19s ± 2m27s |
| the scanner alone (its serial part) | 6m59s ± 39.5s |
| the workers alone, over 4 | 15m19s ± 2m27s |

At 4 threads the workers bound it. Per byte: the scanner 4.0 ns a byte of bz2, the workers 2.01 ns a byte of output at 4 threads (CPU, summed over them), 1.28 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,269,316 | 15,087,510 | \`c5dc29887a3b9d42…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4157.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
