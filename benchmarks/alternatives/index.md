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

The latest record: [`47149bf`](<https://github.com/lmmx/mibz2/commit/47149bff85a13256f36982399d900672ab607254>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42811459401>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.309 s | 678 | 6.3 | 2.79× (2.70–2.89) | 7.07× (6.84–7.31) | 7.51× (7.29–7.74) | 8.47× (8.18–8.78) | 7.71× (7.42–8.01) | 7.79× (7.53–8.06) | 7.37× (7.12–7.64) | 2.22× (2.14–2.30) | 2.56× (2.48–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.201 s | 1,043 | 7.4 | 2.68× (2.50–2.88) | 6.05× (5.69–6.44) | 7.29× (6.82–7.79) | 7.26× (6.83–7.72) | 6.54× (6.15–6.95) | 12.14× (11.44–12.89) | 11.49× (10.83–12.20) | 3.46× (3.26–3.67) | 3.99× (3.74–4.24) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.180 s | 1,165 | 12.9 | 2.48× (2.37–2.59) | 3.98× (3.83–4.14) | 5.23× (4.88–5.60) | 4.97× (4.70–5.25) | 4.87× (4.72–5.03) | 13.38× (12.97–13.81) | 12.67× (12.29–13.06) | 3.81× (3.68–3.95) | 4.39× (4.22–4.57) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,360 | 3.5 | 0.57× (0.55–0.58) | 3.47× (3.41–3.53) | 4.45× (4.24–4.67) | 3.69× (3.60–3.78) | 6.48× (6.33–6.63) | 2.82× (2.73–2.90) | 2.79× (2.72–2.85) | 1.05× (0.94–1.17) | 2.08× (2.05–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,320 | 3.6 | 0.51× (0.50–0.52) | 3.03× (2.77–3.32) | 5.65× (5.40–5.90) | 3.62× (3.51–3.74) | 6.50× (6.14–6.88) | 2.74× (2.66–2.82) | 2.71× (2.64–2.78) | 1.02× (0.92–1.14) | 2.03× (1.98–2.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,351 | 3.5 | 0.52× (0.50–0.53) | 3.08× (2.81–3.38) | 5.84× (5.46–6.25) | 3.68× (3.58–3.78) | 6.42× (6.25–6.59) | 2.77× (2.69–2.86) | 2.74× (2.66–2.83) | 1.04× (0.93–1.15) | 2.05× (2.00–2.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,021 | 4.5 | 2.86× (2.81–2.91) | 3.72× (3.63–3.81) | 4.37× (4.27–4.48) | 4.61× (4.50–4.73) | 5.45× (5.26–5.66) | 3.23× (2.95–3.55) | 2.72× (2.56–2.90) | 2.16× (2.11–2.21) | 3.18× (3.10–3.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,320 | 6.3 | 2.06× (1.92–2.21) | 2.85× (2.64–3.07) | 4.02× (3.68–4.38) | 3.75× (3.22–4.36) | 5.45× (5.07–5.85) | 4.21× (3.98–4.44) | 3.54× (3.40–3.69) | 2.81× (2.61–3.02) | 4.14× (3.86–4.43) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,146 | 12.6 | 1.73× (1.68–1.78) | 1.91× (1.85–1.96) | 2.73× (2.57–2.89) | 2.17× (2.07–2.27) | 4.23× (4.06–4.41) | 3.66× (3.32–4.04) | 3.09× (2.86–3.33) | 2.45× (2.34–2.56) | 3.60× (3.47–3.74) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.082 s | 97 | 44.7 | 1.38× (1.34–1.42) | 3.07× (2.97–3.17) | 2.84× (2.70–2.97) | 2.75× (2.66–2.85) | 3.16× (3.01–3.33) | 3.58× (3.42–3.76) | 3.48× (3.37–3.60) | 1.33× (1.28–1.37) | 1.52× (1.46–1.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.053 s | 150 | 52.1 | 1.26× (1.23–1.30) | 2.86× (2.72–3.01) | 2.66× (2.57–2.76) | 3.22× (3.16–3.29) | 3.11× (3.05–3.17) | 5.58× (5.38–5.79) | 5.42× (5.26–5.59) | 2.07× (2.01–2.13) | 2.36× (2.29–2.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.045 s | 179 | 71.0 | 1.29× (1.18–1.41) | 2.38× (2.32–2.44) | 2.34× (2.24–2.45) | 2.78× (2.72–2.84) | 3.07× (3.02–3.13) | 6.63× (6.39–6.88) | 6.44× (6.30–6.59) | 2.46× (2.39–2.53) | 2.81× (2.74–2.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.088 s | 108 | 40.1 | 1.41× (1.24–1.60) | 3.49× (3.06–3.97) | 2.92× (2.54–3.35) | 2.93× (2.62–3.27) | 3.30× (2.88–3.78) | 4.24× (3.68–4.88) | 4.50× (3.95–5.12) | 1.54× (1.32–1.80) | 1.73× (1.51–1.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.057 s | 169 | 47.7 | 1.33× (1.28–1.38) | 3.27× (3.15–3.39) | 2.78× (2.70–2.86) | 3.77× (3.61–3.93) | 3.39× (3.26–3.52) | 6.97× (6.71–7.24) | 7.39× (7.13–7.67) | 2.54× (2.40–2.68) | 2.85× (2.75–2.96) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.049 s | 197 | 66.2 | 1.42× (1.24–1.62) | 2.62× (2.50–2.76) | 2.48× (2.39–2.58) | 3.31× (3.17–3.46) | 3.15× (3.01–3.30) | 8.04× (7.67–8.44) | 8.54× (8.26–8.82) | 2.93× (2.70–3.17) | 3.29× (3.17–3.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 4.9 | 0.63× (0.59–0.67) | 3.11× (3.02–3.22) | 4.29× (4.15–4.43) | 3.16× (3.04–3.28) | 7.12× (6.88–7.36) | 2.35× (2.30–2.41) | 2.52× (2.15–2.96) | 0.97× (0.94–1.01) | 2.08× (2.02–2.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 976 | 5.2 | 0.52× (0.51–0.53) | 3.03× (2.95–3.10) | 5.19× (5.06–5.33) | 3.08× (3.01–3.15) | 6.76× (6.59–6.93) | 2.28× (2.20–2.37) | 2.45× (2.08–2.88) | 0.94× (0.90–0.98) | 2.02× (1.97–2.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 952 | 5.1 | 0.51× (0.50–0.52) | 2.95× (2.90–3.00) | 5.13× (4.95–5.33) | 3.03× (3.00–3.07) | 6.63× (6.50–6.78) | 2.22× (2.16–2.28) | 2.38× (2.05–2.77) | 0.92× (0.89–0.94) | 1.96× (1.93–2.00) | yes |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 1 | 0.045 s | 283 | 15.6 | 1.67× (1.55–1.80) | 4.08× (3.80–4.38) | 4.41× (4.09–4.76) | 4.65× (4.29–5.03) | 4.82× (4.44–5.23) | 4.90× (4.54–5.30) | 4.68× (4.34–5.04) | 1.45× (1.35–1.57) | 1.72× (1.60–1.86) | yes |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 2 | 0.034 s | 382 | 20.8 | 1.34× (1.24–1.44) | 3.13× (2.95–3.32) | 3.86× (3.56–4.19) | 4.25× (3.98–4.54) | 4.36× (4.06–4.69) | 6.77× (6.48–7.08) | 6.46× (6.15–6.78) | 2.01× (1.92–2.10) | 2.38× (2.29–2.48) | yes |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 4 | 0.031 s | 412 | 32.2 | 1.32× (1.27–1.37) | 2.41× (2.25–2.58) | 3.12× (2.94–3.31) | 3.28× (3.17–3.40) | 3.87× (3.75–3.98) | 7.40× (7.17–7.64) | 7.06× (6.83–7.30) | 2.19× (2.12–2.27) | 2.60× (2.49–2.71) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.78× (2.69–2.88) | 7.02× (6.79–7.26) | 7.46× (7.24–7.68) | 8.43× (8.14–8.74) | 7.67× (7.39–7.97) | 7.75× (7.50–8.02) | 7.34× (7.08–7.60) | 2.21× (2.14–2.29) | 2.53× (2.46–2.61) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.57× (2.47–2.67) | 6.35× (6.15–6.55) | 6.85× (6.49–7.22) | 7.73× (7.47–8.01) | 6.68× (6.45–6.91) | 6.61× (6.40–6.82) | 6.25× (6.06–6.46) | 1.89× (1.83–1.95) | 2.16× (2.07–2.25) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.43× (2.34–2.53) | 4.46× (4.36–4.57) | 5.04× (4.77–5.33) | 5.61× (5.34–5.89) | 4.80× (4.65–4.96) | 3.88× (3.75–4.01) | 3.67× (3.57–3.78) | 1.11× (1.08–1.14) | 1.27× (1.23–1.31) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.63) | 3.05× (2.97–3.12) | 3.89× (3.84–3.94) | 3.41× (3.33–3.49) | 5.91× (5.78–6.04) | 2.64× (2.55–2.74) | 2.60× (2.54–2.66) | 1.06× (0.93–1.21) | 1.82× (1.79–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.58) | 2.81× (2.74–2.88) | 4.96× (4.82–5.10) | 3.33× (3.23–3.43) | 5.86× (5.54–6.19) | 2.53× (2.45–2.62) | 2.50× (2.44–2.55) | 1.02× (0.90–1.16) | 1.74× (1.71–1.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.58) | 3.08× (3.00–3.17) | 5.11× (4.90–5.34) | 3.46× (3.37–3.56) | 5.91× (5.76–6.06) | 2.63× (2.53–2.74) | 2.59× (2.52–2.67) | 1.06× (0.93–1.21) | 1.81× (1.78–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.71× (2.67–2.75) | 3.17× (3.11–3.22) | 3.80× (3.72–3.89) | 4.32× (4.22–4.42) | 5.10× (4.92–5.28) | 3.06× (2.80–3.35) | 2.59× (2.44–2.75) | 2.07× (2.02–2.12) | 2.80× (2.77–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.11× (2.03–2.20) | 2.48× (2.38–2.58) | 3.02× (2.91–3.13) | 3.31× (3.19–3.44) | 3.77× (3.63–3.91) | 2.21× (2.02–2.41) | 1.86× (1.72–2.02) | 1.49× (1.41–1.57) | 2.02× (1.94–2.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.85× (1.76–1.95) | 1.61× (1.51–1.71) | 1.98× (1.88–2.09) | 2.00× (1.89–2.13) | 2.15× (2.03–2.29) | 1.11× (1.00–1.23) | 0.94× (0.86–1.02) | 0.75× (0.71–0.80) | 1.02× (0.96–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.33–1.41) | 2.99× (2.90–3.09) | 2.74× (2.63–2.86) | 2.72× (2.63–2.81) | 3.12× (2.97–3.28) | 3.53× (3.37–3.70) | 3.43× (3.32–3.54) | 1.32× (1.28–1.37) | 1.45× (1.41–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.82× (2.68–2.97) | 2.39× (2.34–2.44) | 3.08× (3.01–3.15) | 2.74× (2.69–2.79) | 3.06× (2.93–3.19) | 2.97× (2.88–3.06) | 1.14× (1.11–1.18) | 1.26× (1.22–1.30) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.44× (1.34–1.55) | 2.58× (2.52–2.64) | 2.11× (2.07–2.16) | 2.80× (2.70–2.90) | 2.63× (2.60–2.67) | 2.27× (2.20–2.35) | 2.21× (2.17–2.25) | 0.85× (0.83–0.87) | 0.94× (0.92–0.95) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.23–1.59) | 3.38× (2.96–3.87) | 2.83× (2.45–3.26) | 2.89× (2.58–3.23) | 3.25× (2.83–3.73) | 4.17× (3.61–4.82) | 4.42× (3.87–5.04) | 1.53× (1.31–1.78) | 1.66× (1.44–1.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.30–1.35) | 3.21× (3.13–3.28) | 2.54× (2.48–2.60) | 3.51× (3.43–3.60) | 3.02× (2.91–3.14) | 3.72× (3.61–3.83) | 3.93× (3.81–4.06) | 1.36× (1.28–1.44) | 1.47× (1.44–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.47–1.71) | 2.91× (2.85–2.96) | 2.19× (2.13–2.24) | 3.11× (3.03–3.20) | 2.87× (2.73–3.01) | 2.69× (2.59–2.79) | 2.84× (2.77–2.92) | 0.98× (0.93–1.05) | 1.07× (1.04–1.09) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.64–0.72) | 2.53× (2.44–2.63) | 3.37× (3.27–3.48) | 2.84× (2.74–2.94) | 6.22× (6.02–6.42) | 2.16× (2.12–2.20) | 2.32× (1.97–2.73) | 0.98× (0.94–1.01) | 1.48× (1.43–1.53) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.37× (2.31–2.42) | 4.01× (3.92–4.10) | 2.79× (2.74–2.85) | 5.83× (5.68–5.97) | 2.06× (1.97–2.15) | 2.21× (1.87–2.60) | 0.93× (0.90–0.97) | 1.41× (1.38–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.52× (2.47–2.57) | 4.11× (3.99–4.23) | 2.85× (2.81–2.88) | 5.94× (5.83–6.05) | 2.08× (2.01–2.15) | 2.23× (1.91–2.60) | 0.94× (0.92–0.96) | 1.42× (1.40–1.45) |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 1 | 1.64× (1.51–1.78) | 3.86× (3.57–4.17) | 4.18× (3.87–4.52) | 4.50× (4.14–4.90) | 4.67× (4.28–5.10) | 4.75× (4.37–5.15) | 4.53× (4.19–4.91) | 1.43× (1.32–1.55) | 1.57× (1.45–1.70) |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 2 | 1.38× (1.35–1.42) | 3.17× (3.10–3.25) | 3.55× (3.38–3.73) | 3.97× (3.87–4.07) | 3.80× (3.64–3.97) | 3.69× (3.61–3.76) | 3.52× (3.43–3.61) | 1.11× (1.09–1.14) | 1.22× (1.18–1.26) |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 4 | 1.41× (1.34–1.48) | 2.65× (2.46–2.85) | 2.85× (2.68–3.04) | 3.11× (3.03–3.20) | 3.10× (3.00–3.21) | 2.36× (2.27–2.46) | 2.26× (2.18–2.34) | 0.71× (0.69–0.74) | 0.78× (0.76–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.37) | 1.05× (1.05–1.06) | 0.84× (0.84–0.85) | 11.92× (11.90–11.94) | 0.28× (0.28–0.28) | 0.30× (0.30–0.30) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.52× (0.46–0.59) | 0.75× (0.74–0.76) | 0.63× (0.62–0.64) | 5.98× (5.92–6.05) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.37–0.38) | 0.53× (0.43–0.64) | 0.67× (0.66–0.68) | 0.58× (0.58–0.59) | 3.88× (3.86–3.90) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.40× (4.38–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.44–0.46) | 1.70× (1.69–1.70) | 0.66× (0.66–0.66) | 8.02× (8.00–8.04) | 0.64× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.69) | 0.65× (0.60–0.70) | 1.60× (1.54–1.67) | 0.80× (0.77–0.83) | 5.81× (5.58–6.05) | 0.46× (0.45–0.48) | 0.49× (0.48–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.60–0.65) | 0.65× (0.61–0.69) | 1.14× (1.11–1.17) | 0.70× (0.68–0.73) | 3.33× (3.24–3.42) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.30× (0.29–0.30) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.74× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.53) | 0.43× (0.41–0.45) | 0.92× (0.88–0.96) | 0.74× (0.71–0.78) | 3.19× (3.06–3.33) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.48) | 0.87× (0.85–0.89) | 0.81× (0.77–0.84) | 2.54× (2.48–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.21× (5.21–5.21) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.37× (0.35–0.40) | 0.85× (0.82–0.88) | 0.72× (0.70–0.74) | 2.92× (2.82–3.02) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.48× (0.45–0.51) | 0.80× (0.78–0.81) | 0.81× (0.78–0.85) | 2.31× (2.26–2.35) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.22× (0.20–0.23) | 1.07× (1.07–1.07) | 0.77× (0.77–0.77) | 5.36× (5.36–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.76–0.77) | 5.31× (5.29–5.33) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.32× (5.31–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.41× (1.41–1.41) | 0.66× (0.66–0.67) | 6.38× (6.37–6.39) | 0.38× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 2 | 39 MB | 0.43× (0.42–0.44) | 0.35× (0.33–0.38) | 1.01× (0.98–1.04) | 0.65× (0.64–0.67) | 3.17× (3.12–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.15× (0.15–0.16) |
| `text` (2.54 MB → 12.8 MB, 5.0×) | 4 | 60 MB | 0.46× (0.45–0.48) | 0.44× (0.40–0.49) | 0.85× (0.83–0.87) | 0.69× (0.66–0.72) | 2.17× (2.12–2.22) | 0.12× (0.12–0.13) | 0.14× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 678 | 1.48 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,043 | 1.73 | 1.17× (1.12–1.23) | 78% | 1.06× (1.05–1.07) | 1.08× (1.05–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 1,165 | 3.02 | 2.00× (1.90–2.10) | 43% | 1.27× (1.25–1.29) | 1.74× (1.69–1.80) |

**Best:** mibz2 at 4 threads (1,165 MB/s); lbzip2 at 4 threads (292 MB/s); pbz2 at 4 threads (468 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.93, mibz2 4t 1.24, lbzip2 1t 0.07, lbzip2 2t 0.29, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 14%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.47 | 0.01 | 0.09 | 0.180 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4269 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 39.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.2% |
| MTF + RLE2 | 1.45 | 20.1% |
| T-vector build | 1.30 | 18.1% |
| walk (the one each build uses) | 3.00 | 41.6% |
| of which: segment assembly | 0.13 | 1.8% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.25 | 3.5% |
| **total** | 7.21 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.298 | 0.711 | (in its total) | 2.35× (2.26–2.44) |  |
| walk | 0.649 | 1.596 | (in its total) | 2.42× (2.31–2.53) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.028 | 0.031 | (in its total) | 1.08× (0.99–1.18) |  |
| inverse BWT, total | 0.976 | 2.341 | 1.314 | 2.36× (2.25–2.46) | 1.34× (1.29–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 45m21s ± 5m22s |
| 4 threads | 26m23s ± 3m47s |
| 4 threads, the sample's overlap gap as a fixed cost | 16m48s ± 2m26s |
| the scanner alone (its serial part) | 9m27s ± 32.9s |
| the workers alone, over 4 | 16m48s ± 2m26s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 2.20 ns a byte of output at 4 threads (CPU, summed over them), 1.18 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,541,924 | 12,763,546 | \`17b70328b4b7a6e5…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4268.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.0 (bench/report.py), 10 rounds, threads 1, 2, 4
