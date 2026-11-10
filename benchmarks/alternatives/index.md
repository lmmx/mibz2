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

The latest record: [`cd7c7b2`](<https://github.com/lmmx/mibz2/commit/cd7c7b2610f8ccdeb9132f188e3d659ed32788ed>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42800679392>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.7.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.711 s | 295 | 10.5 | 2.15× (2.14–2.16) | 4.81× (4.79–4.82) | 5.11× (5.08–5.14) | 5.76× (5.74–5.79) | 5.14× (5.11–5.17) | 5.52× (5.49–5.55) | 5.27× (5.24–5.30) | 1.65× (1.65–1.66) | 1.55× (1.54–1.55) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.390 s | 538 | 11.1 | 2.21× (2.13–2.30) | 4.83× (4.56–5.12) | 5.55× (5.30–5.81) | 5.75× (5.49–6.03) | 5.01× (4.82–5.22) | 9.96× (9.55–10.39) | 9.52× (9.11–9.95) | 2.98× (2.86–3.11) | 2.79× (2.67–2.91) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.302 s | 694 | 17.1 | 2.45× (2.36–2.53) | 4.24× (4.11–4.38) | 4.78× (4.59–4.98) | 4.85× (4.68–5.03) | 4.73× (4.57–4.89) | 12.80× (12.33–13.28) | 12.23× (11.79–12.69) | 3.84× (3.70–3.98) | 3.58× (3.45–3.72) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 985 | 3.4 | 0.58× (0.58–0.59) | 3.31× (3.16–3.46) | 4.48× (4.40–4.57) | 3.81× (3.74–3.87) | 6.25× (6.16–6.35) | 2.81× (2.76–2.87) | 2.81× (2.77–2.86) | 1.12× (1.10–1.14) | 1.02× (1.00–1.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 985 | 3.4 | 0.56× (0.55–0.57) | 3.02× (2.98–3.06) | 5.94× (5.80–6.08) | 3.81× (3.76–3.87) | 6.24× (6.18–6.30) | 2.81× (2.78–2.84) | 2.81× (2.77–2.84) | 1.12× (1.10–1.14) | 1.02× (1.00–1.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 990 | 3.4 | 0.56× (0.55–0.57) | 3.04× (2.98–3.11) | 6.02× (5.90–6.14) | 3.83× (3.76–3.91) | 6.25× (6.11–6.40) | 2.82× (2.78–2.86) | 2.82× (2.78–2.86) | 1.12× (1.10–1.14) | 1.02× (1.00–1.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.039 s | 503 | 6.4 | 2.67× (2.65–2.69) | 3.18× (3.10–3.26) | 3.65× (3.61–3.70) | 4.21× (4.17–4.25) | 4.61× (4.57–4.65) | 3.50× (3.46–3.53) | 2.82× (2.80–2.84) | 1.96× (1.94–1.98) | 2.08× (2.06–2.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 830 | 7.4 | 2.40× (2.28–2.53) | 3.04× (2.82–3.29) | 4.04× (3.81–4.29) | 3.85× (3.70–4.00) | 5.29× (4.95–5.67) | 5.65× (5.42–5.89) | 4.56× (4.37–4.76) | 3.17× (3.03–3.32) | 3.37× (3.21–3.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 852 | 12.8 | 2.13× (2.05–2.20) | 2.16× (2.03–2.29) | 3.12× (3.02–3.23) | 2.61× (2.49–2.75) | 4.45× (4.31–4.59) | 5.86× (5.68–6.05) | 4.73× (4.59–4.87) | 3.29× (3.20–3.39) | 3.49× (3.39–3.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.142 s | 56 | 55.8 | 1.34× (1.33–1.34) | 2.70× (2.68–2.72) | 2.37× (2.33–2.40) | 2.50× (2.48–2.51) | 2.78× (2.76–2.79) | 3.12× (3.10–3.13) | 3.18× (3.16–3.19) | 1.26× (1.26–1.27) | 1.28× (1.27–1.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.084 s | 96 | 60.9 | 1.37× (1.25–1.50) | 2.62× (2.53–2.72) | 2.49× (2.40–2.58) | 3.27× (3.12–3.42) | 3.00× (2.84–3.16) | 5.21× (5.02–5.40) | 5.31× (5.12–5.51) | 2.12× (2.04–2.20) | 2.14× (2.06–2.22) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 113 | 83.6 | 1.40× (1.37–1.42) | 2.47× (2.43–2.52) | 2.35× (2.28–2.43) | 3.00× (2.92–3.08) | 2.93× (2.89–2.96) | 6.29× (6.21–6.38) | 6.42× (6.33–6.52) | 2.56× (2.52–2.59) | 2.58× (2.55–2.62) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.159 s | 60 | 52.0 | 1.37× (1.36–1.38) | 3.06× (3.03–3.10) | 2.52× (2.50–2.54) | 2.70× (2.68–2.72) | 2.99× (2.97–3.01) | 3.77× (3.75–3.78) | 3.95× (3.91–4.00) | 1.48× (1.47–1.48) | 1.53× (1.52–1.54) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.093 s | 103 | 57.0 | 1.35× (1.21–1.50) | 2.88× (2.78–2.98) | 2.56× (2.48–2.64) | 3.65× (3.52–3.79) | 3.06× (2.96–3.15) | 6.38× (6.20–6.56) | 6.70× (6.51–6.89) | 2.50× (2.43–2.57) | 2.59× (2.52–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 79.1 | 1.48× (1.43–1.53) | 2.79× (2.71–2.86) | 2.44× (2.38–2.51) | 3.41× (3.31–3.50) | 3.04× (2.97–3.12) | 7.77× (7.57–7.97) | 8.16× (7.94–8.38) | 3.04× (2.97–3.12) | 3.15× (3.06–3.25) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 4.7 | 0.63× (0.61–0.64) | 3.00× (2.96–3.05) | 3.71× (3.66–3.77) | 3.22× (3.17–3.26) | 6.70× (6.59–6.82) | 2.38× (2.34–2.41) | 2.39× (2.37–2.42) | 1.02× (1.00–1.04) | 1.51× (1.49–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 712 | 5.0 | 0.53× (0.53–0.54) | 2.93× (2.88–2.98) | 5.06× (4.96–5.16) | 3.16× (3.09–3.22) | 6.53× (6.40–6.66) | 2.31× (2.26–2.35) | 2.32× (2.28–2.37) | 0.99× (0.96–1.01) | 1.47× (1.44–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 699 | 4.9 | 0.53× (0.51–0.55) | 2.86× (2.76–2.97) | 4.94× (4.76–5.13) | 3.08× (2.98–3.19) | 6.32× (6.07–6.59) | 2.25× (2.17–2.34) | 2.27× (2.18–2.35) | 0.96× (0.93–1.00) | 1.44× (1.38–1.49) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 0.092 s | 133 | 23.8 | 1.34× (1.34–1.35) | 2.94× (2.90–2.98) | 3.26× (3.23–3.29) | 3.41× (3.40–3.43) | 3.53× (3.49–3.58) | 3.70× (3.66–3.74) | 3.62× (3.60–3.63) | 1.12× (1.10–1.13) | 1.11× (1.10–1.11) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 0.057 s | 214 | 26.4 | 1.26× (1.17–1.36) | 2.66× (2.61–2.72) | 3.22× (3.15–3.30) | 3.80× (3.68–3.93) | 3.61× (3.48–3.74) | 5.99× (5.85–6.12) | 5.85× (5.74–5.97) | 1.80× (1.77–1.84) | 1.79× (1.75–1.82) | yes |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 0.046 s | 267 | 37.8 | 1.35× (1.33–1.38) | 2.41× (2.27–2.57) | 2.88× (2.78–2.98) | 3.51× (3.43–3.58) | 3.53× (3.44–3.63) | 7.41× (7.24–7.59) | 7.25× (7.13–7.37) | 2.23× (2.20–2.27) | 2.22× (2.18–2.25) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.14× (2.14–2.15) | 4.79× (4.77–4.81) | 5.09× (5.06–5.12) | 5.75× (5.73–5.78) | 5.13× (5.10–5.16) | 5.51× (5.47–5.54) | 5.26× (5.23–5.29) | 1.65× (1.65–1.66) | 1.54× (1.53–1.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.14× (2.13–2.14) | 4.87× (4.82–4.93) | 5.29× (5.23–5.35) | 5.93× (5.87–5.99) | 5.02× (5.01–5.04) | 5.26× (5.22–5.29) | 5.02× (5.00–5.05) | 1.58× (1.57–1.58) | 1.47× (1.46–1.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.33× (2.27–2.38) | 4.44× (4.35–4.53) | 4.54× (4.44–4.65) | 5.07× (4.96–5.18) | 4.48× (4.40–4.58) | 3.41× (3.35–3.48) | 3.26× (3.20–3.33) | 1.02× (1.01–1.04) | 0.95× (0.94–0.97) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 2.89× (2.85–2.93) | 3.95× (3.89–4.01) | 3.60× (3.54–3.67) | 5.88× (5.79–5.96) | 2.69× (2.63–2.74) | 2.68× (2.64–2.72) | 1.11× (1.09–1.14) | 0.91× (0.89–0.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.62) | 2.86× (2.82–2.91) | 5.15× (5.08–5.23) | 3.58× (3.53–3.63) | 5.80× (5.75–5.85) | 2.64× (2.62–2.67) | 2.64× (2.61–2.67) | 1.10× (1.08–1.11) | 0.90× (0.88–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.31× (3.25–3.37) | 5.23× (5.12–5.34) | 3.64× (3.56–3.71) | 5.84× (5.71–5.98) | 2.68× (2.64–2.72) | 2.68× (2.64–2.72) | 1.11× (1.09–1.13) | 0.91× (0.89–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.59–2.62) | 3.03× (2.97–3.08) | 3.48× (3.45–3.51) | 4.08× (4.04–4.12) | 4.47× (4.43–4.50) | 3.40× (3.36–3.43) | 2.75× (2.73–2.77) | 1.93× (1.91–1.94) | 1.85× (1.84–1.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.43× (2.40–2.47) | 2.77× (2.73–2.81) | 3.25× (3.20–3.30) | 3.61× (3.57–3.66) | 4.01× (3.87–4.17) | 2.94× (2.89–2.99) | 2.38× (2.34–2.41) | 1.67× (1.64–1.70) | 1.60× (1.58–1.63) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.24× (2.17–2.30) | 1.95× (1.89–2.00) | 2.38× (2.35–2.42) | 2.38× (2.35–2.41) | 2.47× (2.44–2.50) | 1.69× (1.66–1.71) | 1.36× (1.35–1.38) | 0.96× (0.95–0.97) | 0.92× (0.91–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.33–1.34) | 2.64× (2.62–2.65) | 2.33× (2.31–2.35) | 2.48× (2.47–2.49) | 2.76× (2.75–2.77) | 3.09× (3.08–3.11) | 3.15× (3.14–3.17) | 1.26× (1.26–1.27) | 1.25× (1.24–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.30) | 2.60× (2.58–2.63) | 2.30× (2.28–2.32) | 3.07× (3.02–3.12) | 2.69× (2.57–2.80) | 2.84× (2.81–2.86) | 2.89× (2.87–2.91) | 1.16× (1.15–1.17) | 1.14× (1.14–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.51× (1.45–1.57) | 2.65× (2.62–2.69) | 2.18× (2.12–2.23) | 2.93× (2.85–3.00) | 2.58× (2.55–2.60) | 2.07× (2.04–2.10) | 2.11× (2.09–2.14) | 0.85× (0.83–0.86) | 0.84× (0.82–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.37× (1.36–1.37) | 3.01× (2.99–3.04) | 2.47× (2.45–2.50) | 2.68× (2.66–2.70) | 2.97× (2.95–2.99) | 3.74× (3.72–3.76) | 3.92× (3.88–3.96) | 1.47× (1.46–1.48) | 1.48× (1.48–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.31× (1.30–1.32) | 2.92× (2.91–2.94) | 2.41× (2.40–2.42) | 3.40× (3.36–3.44) | 2.81× (2.79–2.83) | 3.40× (3.38–3.42) | 3.57× (3.53–3.61) | 1.34× (1.33–1.35) | 1.35× (1.34–1.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.59–1.67) | 3.04× (2.99–3.09) | 2.21× (2.17–2.26) | 3.16× (3.10–3.22) | 2.74× (2.69–2.79) | 2.46× (2.42–2.50) | 2.58× (2.54–2.63) | 0.97× (0.95–0.98) | 0.98× (0.96–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.68) | 2.38× (2.35–2.41) | 3.37× (3.31–3.43) | 2.98× (2.93–3.03) | 6.10× (5.99–6.22) | 2.23× (2.20–2.27) | 2.24× (2.21–2.27) | 1.02× (1.00–1.05) | 0.82× (0.80–0.83) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.58–0.59) | 2.29× (2.25–2.33) | 4.16× (4.08–4.24) | 2.94× (2.87–3.00) | 5.86× (5.73–5.99) | 2.13× (2.09–2.17) | 2.14× (2.10–2.18) | 0.98× (0.96–1.00) | 0.78× (0.77–0.79) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.60) | 2.62× (2.53–2.71) | 4.16× (4.01–4.31) | 2.95× (2.85–3.05) | 5.82× (5.58–6.07) | 2.13× (2.05–2.22) | 2.14× (2.06–2.22) | 0.98× (0.94–1.01) | 0.78× (0.76–0.80) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 1.34× (1.33–1.34) | 2.86× (2.84–2.87) | 3.18× (3.16–3.19) | 3.37× (3.35–3.39) | 3.49× (3.45–3.53) | 3.65× (3.62–3.69) | 3.57× (3.56–3.58) | 1.11× (1.10–1.13) | 1.05× (1.05–1.06) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 1.28× (1.27–1.29) | 2.72× (2.69–2.75) | 3.15× (3.14–3.17) | 3.51× (3.48–3.53) | 3.28× (3.21–3.35) | 3.27× (3.24–3.30) | 3.20× (3.18–3.21) | 1.00× (0.99–1.01) | 0.94× (0.93–0.95) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 1.56× (1.53–1.59) | 2.63× (2.57–2.69) | 2.89× (2.84–2.95) | 3.15× (3.10–3.20) | 3.01× (2.97–3.05) | 2.30× (2.26–2.34) | 2.25× (2.22–2.28) | 0.70× (0.69–0.71) | 0.66× (0.65–0.67) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.28× (0.28–0.28) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.10× (12.08–12.12) | 0.29× (0.28–0.29) | 0.31× (0.30–0.31) | 0.47× (0.47–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.39× (0.38–0.40) | 0.56× (0.48–0.65) | 0.81× (0.79–0.83) | 0.68× (0.67–0.69) | 6.46× (6.38–6.55) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.25–0.25) | 0.13× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.58× (0.47–0.70) | 0.72× (0.71–0.72) | 0.64× (0.63–0.65) | 4.26× (4.22–4.29) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.41–0.45) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.03× (8.02–8.05) | 0.65× (0.65–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.69) | 0.65× (0.63–0.67) | 1.61× (1.59–1.63) | 0.80× (0.79–0.82) | 5.89× (5.80–5.98) | 0.47× (0.47–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.67) | 0.69× (0.64–0.75) | 1.17× (1.13–1.21) | 0.71× (0.69–0.74) | 3.45× (3.34–3.56) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.31× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.73–0.74) | 5.22× (5.21–5.23) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.50–0.52) | 0.41× (0.39–0.44) | 0.92× (0.91–0.94) | 0.71× (0.68–0.74) | 3.19× (3.13–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.81× (0.79–0.84) | 2.54× (2.47–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.22× (5.22–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.49× (0.47–0.50) | 0.39× (0.38–0.41) | 0.87× (0.84–0.90) | 0.72× (0.68–0.76) | 2.99× (2.89–3.09) | 0.19× (0.18–0.19) | 0.20× (0.19–0.20) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.49) | 0.81× (0.80–0.82) | 0.82× (0.80–0.84) | 2.34× (2.31–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.08× (1.08–1.08) | 0.77× (0.77–0.77) | 5.40× (5.39–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.21) | 1.61× (1.61–1.62) | 0.77× (0.76–0.77) | 5.35× (5.33–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.78) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 1 | 18 MB | 0.48× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.46–1.48) | 0.69× (0.68–0.69) | 6.65× (6.62–6.67) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.33) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 2 | 33 MB | 0.50× (0.49–0.51) | 0.41× (0.37–0.44) | 1.17× (1.13–1.20) | 0.78× (0.74–0.81) | 3.70× (3.59–3.81) | 0.22× (0.22–0.23) | 0.24× (0.23–0.25) | 0.36× (0.35–0.38) | 0.18× (0.18–0.19) |
| `text` (2.42 MB → 12.2 MB, 5.0×) | 4 | 51 MB | 0.55× (0.53–0.56) | 0.50× (0.46–0.55) | 1.00× (0.97–1.03) | 0.78× (0.75–0.82) | 2.55× (2.48–2.62) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 295 | 3.40 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 538 | 3.57 | 1.05× (1.04–1.05) | 90% | 1.07× (1.05–1.08) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 694 | 5.53 | 1.61× (1.58–1.64) | 58% | 1.49× (1.48–1.51) | 1.75× (1.72–1.78) |

**Best:** mibz2 at 4 threads (694 MB/s); lbzip2 at 4 threads (162 MB/s); pbz2 at 4 threads (280 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.72, mibz2 4t 1.07, lbzip2 1t 0.04, lbzip2 2t 0.28, lbzip2 4t 0.29, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 8%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.94 | 0.02 | 0.09 | 0.302 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3101 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.19 | 11.7% |
| MTF + RLE2 | 1.76 | 17.4% |
| T-vector build | 1.58 | 15.6% |
| walk (the one each build uses) | 5.15 | 50.9% |
| of which: segment assembly | 0.27 | 2.7% |
| RLE1 | 0.28 | 2.8% |
| CRC | 0.14 | 1.3% |
| **total** | 10.12 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.490 | 1.274 | (in its total) | 2.60× (2.59–2.61) |  |
| walk | 1.551 | 2.975 | (in its total) | 1.92× (1.91–1.92) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.087 | 0.081 | (in its total) | 0.93× (0.92–0.93) |  |
| inverse BWT, total | 2.128 | 4.328 | 2.330 | 2.04× (2.03–2.04) | 1.09× (1.09–1.10) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h43m ± 12m16s |
| 4 threads | 44m42s ± 6m27s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m45s ± 4m49s |
| the scanner alone (its serial part) | 16m09s ± 19.7s |
| the workers alone, over 4 | 33m45s ± 4m49s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 4.42 ns a byte of output at 4 threads (CPU, summed over them), 2.87 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,422,940 | 12,196,256 | \`61fa7d2de138b6a9…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3100.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.7.0 (bench/report.py), 10 rounds, threads 1, 2, 4
