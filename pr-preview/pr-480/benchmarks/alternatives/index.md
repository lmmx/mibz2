# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-480/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`371e0f3`](<https://github.com/lmmx/mibz2/commit/371e0f3e5842899eca96b0d31069edbef21dc4fd>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43846259045>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.417 s | 503 | 7.3 | 2.94× (2.91–2.96) | 6.77× (6.70–6.84) | 6.97× (6.91–7.04) | 7.93× (7.86–8.01) | 6.94× (6.87–7.00) | 7.29× (7.23–7.35) | 6.99× (6.92–7.05) | 2.17× (2.15–2.19) | 2.43× (2.41–2.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.252 s | 832 | 8.0 | 2.79× (2.64–2.94) | 5.71× (5.34–6.11) | 6.61× (6.24–7.01) | 6.87× (6.45–7.31) | 6.06× (5.66–6.50) | 11.83× (11.04–12.67) | 11.33× (10.60–12.12) | 3.52× (3.28–3.77) | 3.94× (3.68–4.21) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.196 s | 1,070 | 12.2 | 2.95× (2.88–3.03) | 4.39× (4.26–4.52) | 5.48× (5.34–5.62) | 5.38× (5.26–5.50) | 5.23× (5.15–5.31) | 15.38× (15.11–15.65) | 14.74× (14.50–14.98) | 4.58× (4.49–4.67) | 5.12× (5.04–5.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,117 | 3.6 | 0.57× (0.57–0.58) | 3.44× (3.39–3.50) | 4.60× (4.52–4.68) | 3.59× (3.54–3.65) | 6.12× (6.00–6.24) | 2.68× (2.63–2.72) | 2.67× (2.63–2.72) | 0.97× (0.94–1.00) | 1.88× (1.71–2.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,120 | 3.7 | 0.53× (0.52–0.55) | 2.85× (2.77–2.92) | 5.70× (5.55–5.86) | 3.61× (3.48–3.75) | 5.94× (5.79–6.09) | 2.65× (2.57–2.72) | 2.64× (2.57–2.71) | 0.96× (0.93–0.99) | 1.86× (1.68–2.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,090 | 3.7 | 0.53× (0.52–0.54) | 2.83× (2.76–2.89) | 5.64× (5.51–5.78) | 3.52× (3.45–3.60) | 5.95× (5.83–6.06) | 2.61× (2.55–2.68) | 2.61× (2.55–2.67) | 0.95× (0.92–0.97) | 1.84× (1.68–2.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 752 | 5.2 | 2.75× (2.73–2.78) | 3.20× (3.03–3.39) | 3.87× (3.79–3.96) | 4.17× (4.15–4.19) | 4.55× (4.50–4.60) | 2.82× (2.79–2.84) | 2.31× (2.29–2.33) | 2.12× (2.11–2.13) | 3.11× (3.09–3.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,167 | 6.3 | 2.45× (2.33–2.59) | 3.11× (2.93–3.30) | 4.49× (4.27–4.71) | 3.84× (3.67–4.02) | 5.42× (5.23–5.62) | 4.36× (4.22–4.51) | 3.58× (3.46–3.71) | 3.29× (3.18–3.40) | 4.83× (4.67–4.99) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 973 | 12.8 | 1.84× (1.78–1.91) | 1.86× (1.71–2.03) | 2.86× (2.69–3.04) | 2.15× (2.05–2.25) | 4.04× (3.83–4.26) | 3.60× (3.44–3.77) | 2.96× (2.82–3.10) | 2.72× (2.60–2.84) | 3.99× (3.81–4.17) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 51.8 | 1.37× (1.33–1.41) | 2.61× (2.52–2.69) | 2.38× (2.31–2.45) | 2.45× (2.38–2.53) | 2.75× (2.67–2.83) | 2.95× (2.87–3.03) | 3.09× (3.00–3.19) | 1.23× (1.19–1.26) | 1.39× (1.33–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 120 | 57.5 | 1.38× (1.19–1.58) | 2.56× (2.42–2.72) | 2.37× (2.29–2.45) | 3.02× (2.96–3.09) | 2.89× (2.82–2.96) | 4.89× (4.76–5.01) | 5.12× (5.00–5.24) | 2.03× (1.98–2.08) | 2.30× (2.24–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 144 | 77.6 | 1.34× (1.32–1.36) | 2.13× (2.06–2.21) | 2.17× (2.08–2.25) | 2.61× (2.53–2.69) | 2.92× (2.86–2.97) | 5.96× (5.86–6.05) | 6.24× (6.14–6.34) | 2.47× (2.43–2.52) | 2.80× (2.74–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 45.8 | 1.54× (1.53–1.55) | 3.10× (3.06–3.13) | 2.68× (2.64–2.72) | 2.79× (2.77–2.80) | 3.21× (3.07–3.36) | 3.73× (3.70–3.76) | 4.10× (4.07–4.14) | 1.51× (1.50–1.52) | 1.81× (1.79–1.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 131 | 52.0 | 1.36× (1.32–1.40) | 2.83× (2.62–3.04) | 2.51× (2.44–2.58) | 3.42× (3.32–3.51) | 3.05× (2.97–3.13) | 6.00× (5.84–6.17) | 6.60× (6.45–6.75) | 2.43× (2.36–2.50) | 2.91× (2.83–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 150 | 72.1 | 1.38× (1.36–1.41) | 2.16× (2.07–2.25) | 2.19× (2.10–2.28) | 2.91× (2.82–3.00) | 2.79× (2.71–2.88) | 6.90× (6.74–7.06) | 7.59× (7.39–7.80) | 2.79× (2.72–2.86) | 3.34× (3.25–3.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 4.9 | 0.63× (0.62–0.64) | 3.52× (3.48–3.56) | 4.42× (4.37–4.47) | 3.13× (3.11–3.16) | 6.80× (6.65–6.94) | 2.31× (2.29–2.33) | 2.36× (2.33–2.39) | 0.97× (0.96–0.98) | 1.77× (1.75–1.79) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 813 | 5.3 | 0.52× (0.51–0.53) | 2.50× (2.44–2.56) | 5.02× (4.91–5.14) | 2.97× (2.91–3.03) | 6.32× (6.19–6.46) | 2.18× (2.13–2.23) | 2.23× (2.18–2.28) | 0.91× (0.89–0.93) | 1.67× (1.63–1.71) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 5.2 | 0.53× (0.52–0.53) | 2.50× (2.46–2.54) | 5.00× (4.92–5.08) | 2.97× (2.93–3.02) | 6.32× (6.20–6.45) | 2.17× (2.14–2.20) | 2.22× (2.18–2.25) | 0.91× (0.90–0.92) | 1.66× (1.63–1.69) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.057 s | 233 | 16.3 | 1.84× (1.82–1.85) | 4.05× (4.02–4.08) | 4.40× (4.37–4.43) | 4.63× (4.60–4.65) | 4.83× (4.75–4.91) | 4.77× (4.74–4.81) | 4.81× (4.78–4.85) | 1.51× (1.50–1.52) | 1.76× (1.74–1.77) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.039 s | 340 | 20.2 | 1.56× (1.50–1.62) | 3.34× (3.24–3.44) | 4.04× (3.86–4.22) | 4.42× (4.30–4.54) | 4.50× (4.38–4.62) | 7.08× (6.91–7.26) | 7.14× (6.97–7.32) | 2.24× (2.18–2.30) | 2.61× (2.54–2.67) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.035 s | 382 | 31.4 | 1.52× (1.48–1.57) | 2.63× (2.55–2.71) | 3.23× (3.12–3.34) | 3.46× (3.35–3.58) | 4.04× (3.95–4.13) | 7.91× (7.69–8.14) | 7.98× (7.77–8.20) | 2.50× (2.44–2.57) | 2.91× (2.83–3.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.93× (2.90–2.95) | 6.74× (6.67–6.80) | 6.94× (6.88–7.00) | 7.90× (7.83–7.98) | 6.91× (6.85–6.98) | 7.27× (7.21–7.33) | 6.96× (6.90–7.03) | 2.17× (2.14–2.19) | 2.41× (2.39–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.82× (2.78–2.87) | 6.34× (6.29–6.39) | 6.61× (6.56–6.65) | 7.62× (7.57–7.67) | 6.46× (6.40–6.52) | 6.69× (6.64–6.74) | 6.41× (6.37–6.45) | 1.99× (1.97–2.01) | 2.22× (2.20–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.91× (2.84–2.98) | 4.86× (4.80–4.92) | 5.26× (5.14–5.38) | 5.90× (5.82–5.98) | 5.11× (5.00–5.22) | 4.38× (4.33–4.44) | 4.20× (4.14–4.26) | 1.31× (1.29–1.32) | 1.45× (1.43–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.93× (2.89–2.98) | 3.92× (3.88–3.96) | 3.37× (3.32–3.42) | 5.67× (5.57–5.77) | 2.53× (2.49–2.57) | 2.53× (2.49–2.57) | 0.97× (0.95–1.00) | 1.65× (1.63–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.74× (2.68–2.81) | 4.94× (4.82–5.06) | 3.35× (3.24–3.47) | 5.43× (5.31–5.56) | 2.46× (2.40–2.52) | 2.45× (2.40–2.51) | 0.94× (0.92–0.97) | 1.61× (1.57–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.01× (2.95–3.06) | 4.94× (4.86–5.02) | 3.35× (3.29–3.41) | 5.53× (5.43–5.64) | 2.49× (2.43–2.54) | 2.48× (2.43–2.54) | 0.95× (0.93–0.98) | 1.62× (1.59–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.62–2.66) | 2.93× (2.81–3.06) | 3.41× (3.40–3.43) | 3.97× (3.95–4.00) | 4.33× (4.28–4.38) | 2.70× (2.68–2.73) | 2.23× (2.21–2.25) | 2.05× (2.04–2.06) | 2.67× (2.65–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.36× (2.31–2.41) | 2.57× (2.50–2.65) | 3.43× (3.36–3.50) | 3.43× (3.35–3.51) | 3.73× (3.65–3.82) | 2.23× (2.17–2.29) | 1.84× (1.79–1.89) | 1.69× (1.65–1.73) | 2.20× (2.15–2.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.06× (2.02–2.11) | 1.70× (1.65–1.76) | 2.27× (2.21–2.33) | 2.10× (2.04–2.17) | 2.17× (2.10–2.23) | 1.11× (1.08–1.15) | 0.92× (0.89–0.95) | 0.85× (0.82–0.87) | 1.10× (1.07–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.32–1.40) | 2.53× (2.46–2.61) | 2.33× (2.26–2.39) | 2.43× (2.36–2.51) | 2.72× (2.65–2.80) | 2.92× (2.84–3.00) | 3.06× (2.97–3.15) | 1.22× (1.19–1.26) | 1.35× (1.31–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.31) | 2.44× (2.41–2.47) | 2.17× (2.14–2.19) | 2.88× (2.83–2.94) | 2.53× (2.49–2.56) | 2.65× (2.62–2.68) | 2.77× (2.74–2.80) | 1.11× (1.10–1.12) | 1.22× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.44–1.47) | 2.27× (2.23–2.31) | 1.91× (1.88–1.93) | 2.52× (2.50–2.55) | 2.47× (2.42–2.51) | 1.97× (1.95–2.00) | 2.06× (2.04–2.09) | 0.82× (0.81–0.84) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.52–1.54) | 3.01× (2.99–3.03) | 2.61× (2.58–2.64) | 2.76× (2.74–2.78) | 3.18× (3.04–3.33) | 3.69× (3.66–3.72) | 4.06× (4.02–4.09) | 1.50× (1.49–1.51) | 1.74× (1.73–1.75) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.36–1.42) | 2.76× (2.69–2.82) | 2.36× (2.32–2.40) | 3.22× (3.15–3.29) | 2.76× (2.70–2.82) | 3.23× (3.16–3.30) | 3.55× (3.49–3.61) | 1.31× (1.28–1.35) | 1.52× (1.49–1.56) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.58–1.68) | 2.54× (2.49–2.59) | 2.02× (1.99–2.06) | 2.82× (2.73–2.92) | 2.63× (2.56–2.70) | 2.34× (2.28–2.41) | 2.57× (2.51–2.64) | 0.95× (0.93–0.97) | 1.10× (1.08–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.49× (2.46–2.52) | 3.46× (3.42–3.51) | 2.86× (2.84–2.89) | 6.07× (5.94–6.19) | 2.15× (2.13–2.17) | 2.19× (2.16–2.21) | 0.97× (0.96–0.98) | 1.39× (1.37–1.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.26× (2.22–2.30) | 3.99× (3.94–4.03) | 2.73× (2.68–2.78) | 5.57× (5.47–5.67) | 1.99× (1.95–2.03) | 2.03× (1.99–2.07) | 0.90× (0.89–0.92) | 1.29× (1.27–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.50× (2.45–2.54) | 4.04× (3.98–4.11) | 2.81× (2.77–2.85) | 5.73× (5.62–5.84) | 2.04× (2.01–2.07) | 2.08× (2.04–2.11) | 0.92× (0.91–0.94) | 1.32× (1.30–1.33) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.81× (1.80–1.83) | 3.84× (3.80–3.88) | 4.22× (4.20–4.24) | 4.52× (4.50–4.55) | 4.72× (4.64–4.81) | 4.67× (4.63–4.70) | 4.70× (4.67–4.74) | 1.50× (1.49–1.50) | 1.64× (1.63–1.65) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.58× (1.53–1.62) | 3.24× (3.20–3.29) | 3.63× (3.54–3.71) | 4.16× (4.10–4.23) | 3.88× (3.82–3.94) | 3.78× (3.72–3.85) | 3.81× (3.75–3.88) | 1.21× (1.19–1.23) | 1.33× (1.31–1.36) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.65× (1.62–1.68) | 2.59× (2.55–2.64) | 2.82× (2.77–2.88) | 3.23× (3.16–3.30) | 3.06× (3.01–3.11) | 2.41× (2.35–2.48) | 2.43× (2.37–2.50) | 0.77× (0.76–0.79) | 0.85× (0.83–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.79× (0.78–0.80) | 11.05× (11.03–11.07) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.47× (0.41–0.54) | 0.68× (0.65–0.70) | 0.57× (0.55–0.59) | 5.39× (5.22–5.56) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.33× (0.32–0.35) | 0.51× (0.42–0.62) | 0.59× (0.56–0.62) | 0.52× (0.49–0.54) | 3.45× (3.29–3.61) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.65× (0.65–0.66) | 8.04× (8.00–8.08) | 0.65× (0.65–0.65) | 0.69× (0.68–0.70) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.63× (0.57–0.69) | 1.60× (1.54–1.67) | 0.81× (0.77–0.84) | 5.81× (5.57–6.06) | 0.47× (0.45–0.49) | 0.50× (0.48–0.51) | 0.54× (0.52–0.56) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.61–0.65) | 0.67× (0.61–0.72) | 1.12× (1.08–1.17) | 0.70× (0.67–0.72) | 3.34× (3.21–3.48) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.36) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.46–0.52) | 0.40× (0.37–0.43) | 0.89× (0.85–0.93) | 0.71× (0.69–0.74) | 3.08× (2.95–3.22) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.85× (0.83–0.88) | 0.80× (0.77–0.83) | 2.49× (2.43–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.23× (5.22–5.24) | 0.32× (0.32–0.32) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.48) | 0.37× (0.35–0.39) | 0.83× (0.80–0.87) | 0.69× (0.67–0.71) | 2.86× (2.76–2.97) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.60–0.63) | 0.48× (0.45–0.51) | 0.82× (0.80–0.83) | 0.82× (0.80–0.84) | 2.36× (2.31–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.77) | 5.33× (5.31–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.68× (0.67–0.68) | 6.42× (6.40–6.45) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.41× (0.38–0.44) | 1.04× (1.01–1.08) | 0.68× (0.67–0.70) | 3.33× (3.26–3.40) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 57 MB | 0.48× (0.47–0.49) | 0.43× (0.39–0.48) | 0.88× (0.86–0.90) | 0.74× (0.72–0.76) | 2.25× (2.20–2.31) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.21–0.22) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 503 | 2.00 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 832 | 2.16 | 1.09× (1.07–1.10) | 81% | 1.02× (1.01–1.03) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,070 | 3.31 | 1.66× (1.63–1.69) | 53% | 1.20× (1.18–1.21) | 1.65× (1.59–1.70) |

**Best:** mibz2 at 4 threads (1,070 MB/s); lbzip2 at 4 threads (242 MB/s); pbz2 at 4 threads (362 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.88, mibz2 4t 1.22, lbzip2 1t 0.06, lbzip2 2t 0.28, lbzip2 4t 0.43, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.53 | 0.02 | 0.08 | 0.196 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3683 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.10 | 12.8% |
| MTF + RLE2 | 1.57 | 18.2% |
| T-vector build | 1.47 | 17.0% |
| walk (the one each build uses) | 4.02 | 46.7% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.25 | 2.9% |
| **total** | 8.62 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.390 | 1.063 | (in its total) | 2.73× (2.72–2.74) |  |
| walk | 1.040 | 2.060 | (in its total) | 1.99× (1.98–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.040 | (in its total) | 1.11× (1.11–1.12) |  |
| inverse BWT, total | 1.466 | 3.164 | 1.883 | 2.16× (2.15–2.17) | 1.28× (1.28–1.29) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h00m ± 6m59s |
| 4 threads | 28m47s ± 4m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m04s ± 2m42s |
| the scanner alone (its serial part) | 10m55s ± 32.9s |
| the workers alone, over 4 | 19m04s ± 2m42s |

At 4 threads the workers bound it. Per byte: the scanner 6.3 ns a byte of bz2, the workers 2.50 ns a byte of output at 4 threads (CPU, summed over them), 1.63 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,453 | 13,353,938 | \`724a67aca091a7ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3683.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
