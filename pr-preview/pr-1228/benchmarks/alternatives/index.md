# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1228/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`5dd7358`](<https://github.com/lmmx/mibz2/commit/5dd73589fcd58c72082011a80c72f57c9a9b95ab>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46070952752>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.382 s | 549 | 6.8 | 3.15× (3.12–3.17) | 7.23× (7.18–7.29) | 7.46× (7.40–7.52) | 8.51× (8.44–8.58) | 7.48× (7.41–7.54) | 7.84× (7.77–7.90) | 7.56× (7.51–7.62) | 2.34× (2.32–2.36) | 2.63× (2.61–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.213 s | 986 | 7.2 | 3.38× (3.16–3.62) | 6.72× (6.54–6.91) | 7.81× (7.58–8.05) | 8.14× (7.87–8.42) | 7.13× (6.96–7.31) | 13.94× (13.60–14.29) | 13.46× (13.11–13.81) | 4.16× (4.06–4.27) | 4.68× (4.56–4.81) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.178 s | 1,175 | 11.5 | 3.21× (3.18–3.25) | 4.80× (4.71–4.88) | 5.90× (5.80–6.00) | 5.77× (5.70–5.83) | 5.68× (5.62–5.74) | 16.77× (16.58–16.95) | 16.19× (16.01–16.37) | 5.01× (4.95–5.06) | 5.63× (5.56–5.70) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,117 | 3.6 | 0.57× (0.56–0.58) | 3.41× (3.37–3.44) | 4.55× (4.51–4.60) | 3.54× (3.51–3.57) | 6.00× (5.92–6.08) | 2.64× (2.62–2.66) | 2.64× (2.62–2.67) | 0.94× (0.92–0.97) | 1.91× (1.73–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,081 | 3.9 | 0.51× (0.50–0.53) | 2.76× (2.72–2.81) | 5.54× (5.45–5.65) | 3.45× (3.40–3.50) | 5.75× (5.67–5.83) | 2.56× (2.52–2.60) | 2.56× (2.52–2.60) | 0.91× (0.89–0.94) | 1.86× (1.67–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,050 | 3.9 | 0.51× (0.50–0.52) | 2.70× (2.66–2.74) | 5.44× (5.31–5.58) | 3.35× (3.30–3.41) | 5.68× (5.60–5.76) | 2.49× (2.45–2.53) | 2.49× (2.46–2.53) | 0.89× (0.87–0.91) | 1.81× (1.63–2.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 775 | 5.1 | 2.84× (2.82–2.86) | 3.23× (3.21–3.25) | 4.00× (3.92–4.09) | 4.30× (4.28–4.33) | 4.68× (4.65–4.71) | 2.92× (2.88–2.96) | 2.38× (2.37–2.40) | 2.18× (2.17–2.19) | 3.22× (3.20–3.24) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,229 | 6.0 | 2.47× (2.26–2.70) | 3.12× (2.91–3.36) | 4.65× (4.14–5.23) | 3.99× (3.69–4.31) | 5.52× (5.15–5.91) | 4.50× (4.22–4.79) | 3.67× (3.41–3.94) | 3.36× (3.12–3.61) | 4.95× (4.60–5.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,104 | 12.0 | 2.07× (1.96–2.19) | 2.01× (1.87–2.16) | 3.21× (2.92–3.54) | 2.46× (2.30–2.62) | 4.51× (4.25–4.78) | 4.09× (3.83–4.37) | 3.33× (3.14–3.54) | 3.05× (2.87–3.25) | 4.50× (4.23–4.78) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.101 s | 79 | 47.6 | 1.49× (1.48–1.50) | 2.83× (2.80–2.87) | 2.61× (2.59–2.64) | 2.70× (2.69–2.72) | 3.03× (3.01–3.04) | 3.24× (3.23–3.25) | 3.43× (3.42–3.43) | 1.36× (1.35–1.36) | 1.51× (1.49–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.063 s | 127 | 52.9 | 1.39× (1.31–1.48) | 2.61× (2.46–2.77) | 2.53× (2.40–2.66) | 3.28× (3.12–3.45) | 3.06× (2.90–3.24) | 5.19× (4.92–5.47) | 5.48× (5.19–5.79) | 2.17× (2.05–2.30) | 2.42× (2.28–2.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 163 | 68.3 | 1.49× (1.45–1.54) | 2.43× (2.30–2.57) | 2.36× (2.28–2.45) | 2.98× (2.88–3.07) | 3.23× (3.13–3.33) | 6.64× (6.44–6.85) | 7.02× (6.81–7.24) | 2.78× (2.70–2.87) | 3.10× (2.99–3.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.109 s | 88 | 42.8 | 1.61× (1.56–1.67) | 3.22× (3.10–3.34) | 2.83× (2.73–2.94) | 2.95× (2.85–3.06) | 3.30× (3.19–3.42) | 3.93× (3.80–4.08) | 4.36× (4.21–4.52) | 1.59× (1.54–1.65) | 1.91× (1.84–1.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.067 s | 144 | 48.1 | 1.45× (1.37–1.53) | 2.92× (2.74–3.10) | 2.70× (2.56–2.86) | 3.77× (3.56–3.99) | 3.27× (3.09–3.45) | 6.41× (6.06–6.78) | 7.11× (6.72–7.52) | 2.59× (2.45–2.74) | 3.11× (2.95–3.29) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 176 | 66.0 | 1.63× (1.60–1.67) | 2.62× (2.53–2.70) | 2.61× (2.54–2.68) | 3.52× (3.45–3.59) | 3.25× (3.20–3.31) | 8.04× (7.91–8.18) | 8.92× (8.79–9.05) | 3.25× (3.20–3.30) | 3.91× (3.85–3.96) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 4.9 | 0.64× (0.63–0.65) | 3.52× (3.50–3.54) | 4.36× (4.19–4.53) | 3.14× (3.12–3.16) | 6.72× (6.60–6.83) | 2.33× (2.29–2.36) | 2.37× (2.35–2.38) | 0.96× (0.94–0.97) | 1.77× (1.76–1.78) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 794 | 5.6 | 0.51× (0.51–0.52) | 2.45× (2.43–2.47) | 4.92× (4.88–4.97) | 2.91× (2.88–2.94) | 6.17× (6.05–6.29) | 2.15× (2.12–2.17) | 2.18× (2.16–2.21) | 0.88× (0.87–0.89) | 1.63× (1.62–1.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 784 | 5.5 | 0.51× (0.50–0.52) | 2.43× (2.41–2.45) | 4.87× (4.83–4.91) | 2.89× (2.87–2.92) | 6.15× (6.07–6.23) | 2.12× (2.08–2.16) | 2.16× (2.14–2.18) | 0.87× (0.85–0.89) | 1.62× (1.60–1.63) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 0.051 s | 290 | 13.2 | 2.03× (2.02–2.05) | 4.17× (4.14–4.20) | 4.55× (4.52–4.59) | 4.89× (4.85–4.92) | 5.04× (5.00–5.08) | 4.81× (4.76–4.85) | 4.94× (4.90–4.98) | 1.66× (1.65–1.67) | 1.98× (1.97–1.99) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 0.035 s | 425 | 16.6 | 1.59× (1.53–1.66) | 3.43× (3.29–3.58) | 4.14× (3.98–4.31) | 4.73× (4.51–4.97) | 4.51× (4.38–4.64) | 6.96× (6.76–7.16) | 7.15× (6.94–7.37) | 2.41× (2.34–2.48) | 2.87× (2.78–2.95) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 0.031 s | 475 | 25.5 | 1.63× (1.56–1.70) | 2.54× (2.41–2.68) | 3.45× (3.34–3.57) | 3.76× (3.65–3.87) | 4.14× (4.01–4.28) | 7.86× (7.61–8.12) | 8.08× (7.82–8.35) | 2.72× (2.63–2.81) | 3.24× (3.14–3.35) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.11–3.16) | 7.19× (7.14–7.25) | 7.42× (7.36–7.48) | 8.48× (8.41–8.54) | 7.45× (7.39–7.51) | 7.80× (7.74–7.87) | 7.53× (7.49–7.58) | 2.33× (2.31–2.35) | 2.61× (2.59–2.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.01–3.09) | 6.86× (6.82–6.90) | 7.16× (7.13–7.20) | 8.36× (8.33–8.40) | 7.03× (7.00–7.07) | 7.30× (7.27–7.32) | 7.04× (7.02–7.07) | 2.18× (2.17–2.19) | 2.44× (2.42–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.08× (3.03–3.12) | 5.11× (5.07–5.15) | 5.47× (5.43–5.50) | 6.18× (6.14–6.21) | 5.33× (5.32–5.35) | 4.61× (4.59–4.64) | 4.45× (4.43–4.47) | 1.38× (1.37–1.39) | 1.54× (1.53–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.60–0.61) | 2.90× (2.86–2.95) | 3.83× (3.80–3.86) | 3.32× (3.29–3.35) | 5.56× (5.49–5.63) | 2.49× (2.47–2.52) | 2.50× (2.47–2.52) | 0.95× (0.93–0.97) | 1.63× (1.62–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.56) | 2.61× (2.58–2.65) | 4.66× (4.61–4.72) | 3.14× (3.10–3.18) | 5.16× (5.10–5.23) | 2.34× (2.30–2.37) | 2.34× (2.31–2.37) | 0.89× (0.86–0.91) | 1.53× (1.51–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.54) | 2.84× (2.78–2.89) | 4.69× (4.55–4.82) | 3.12× (3.07–3.18) | 5.19× (5.11–5.26) | 2.32× (2.29–2.36) | 2.33× (2.29–2.36) | 0.88× (0.86–0.90) | 1.52× (1.50–1.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.70–2.74) | 2.97× (2.95–2.98) | 3.53× (3.47–3.58) | 4.10× (4.07–4.12) | 4.45× (4.42–4.48) | 2.80× (2.77–2.84) | 2.30× (2.28–2.31) | 2.11× (2.10–2.12) | 2.76× (2.75–2.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.48× (2.45–2.52) | 2.75× (2.72–2.78) | 3.63× (3.58–3.67) | 3.65× (3.59–3.71) | 3.94× (3.88–4.00) | 2.37× (2.33–2.41) | 1.95× (1.92–1.97) | 1.79× (1.76–1.81) | 2.34× (2.31–2.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.17× (2.13–2.20) | 1.83× (1.79–1.88) | 2.41× (2.34–2.48) | 2.25× (2.19–2.31) | 2.29× (2.25–2.34) | 1.20× (1.17–1.23) | 0.98× (0.96–1.00) | 0.90× (0.88–0.92) | 1.18× (1.15–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.48× (1.48–1.49) | 2.78× (2.77–2.79) | 2.55× (2.52–2.58) | 2.68× (2.66–2.69) | 2.99× (2.98–3.01) | 3.20× (3.19–3.22) | 3.39× (3.38–3.40) | 1.35× (1.35–1.36) | 1.49× (1.48–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.37–1.41) | 2.62× (2.58–2.67) | 2.33× (2.30–2.37) | 3.17× (3.11–3.22) | 2.73× (2.68–2.78) | 2.86× (2.82–2.91) | 3.02× (2.97–3.07) | 1.21× (1.19–1.23) | 1.33× (1.30–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.58–1.67) | 2.53× (2.48–2.59) | 2.16× (2.13–2.19) | 2.95× (2.92–2.98) | 2.77× (2.74–2.79) | 2.23× (2.20–2.26) | 2.36× (2.33–2.39) | 0.94× (0.93–0.96) | 1.04× (1.02–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.60× (1.55–1.66) | 3.16× (3.05–3.27) | 2.75× (2.65–2.86) | 2.92× (2.82–3.03) | 3.27× (3.16–3.38) | 3.89× (3.75–4.03) | 4.31× (4.16–4.47) | 1.58× (1.53–1.64) | 1.83× (1.77–1.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.42–1.55) | 2.95× (2.82–3.08) | 2.52× (2.41–2.64) | 3.56× (3.41–3.72) | 2.96× (2.83–3.09) | 3.45× (3.30–3.60) | 3.82× (3.66–3.99) | 1.40× (1.34–1.46) | 1.63× (1.56–1.70) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.80× (1.78–1.83) | 2.81× (2.79–2.83) | 2.21× (2.17–2.25) | 3.16× (3.11–3.22) | 2.88× (2.85–2.91) | 2.56× (2.54–2.58) | 2.84× (2.82–2.86) | 1.04× (1.03–1.05) | 1.21× (1.20–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.49× (2.47–2.52) | 3.44× (3.38–3.51) | 2.86× (2.84–2.88) | 5.98× (5.88–6.09) | 2.16× (2.12–2.19) | 2.19× (2.17–2.21) | 0.96× (0.95–0.98) | 1.40× (1.38–1.41) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.56–0.57) | 2.16× (2.14–2.18) | 3.76× (3.73–3.79) | 2.61× (2.59–2.64) | 5.32× (5.22–5.41) | 1.92× (1.88–1.95) | 1.94× (1.92–1.96) | 0.86× (0.85–0.87) | 1.24× (1.22–1.26) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.55) | 2.39× (2.36–2.42) | 3.77× (3.75–3.79) | 2.68× (2.66–2.71) | 5.46× (5.39–5.53) | 1.95× (1.91–2.00) | 1.98× (1.96–2.00) | 0.87× (0.86–0.89) | 1.26× (1.25–1.28) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 2.00× (1.99–2.02) | 3.95× (3.92–3.98) | 4.39× (4.36–4.41) | 4.76× (4.73–4.79) | 4.91× (4.87–4.95) | 4.68× (4.64–4.72) | 4.81× (4.78–4.85) | 1.64× (1.63–1.65) | 1.83× (1.81–1.84) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 1.67× (1.65–1.69) | 3.28× (3.24–3.32) | 3.65× (3.61–3.68) | 4.31× (4.26–4.36) | 3.97× (3.93–4.01) | 3.71× (3.67–3.75) | 3.81× (3.77–3.86) | 1.30× (1.28–1.32) | 1.45× (1.43–1.46) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 1.72× (1.67–1.78) | 2.59× (2.55–2.64) | 2.91× (2.86–2.96) | 3.34× (3.29–3.39) | 3.17× (3.11–3.22) | 2.42× (2.38–2.46) | 2.49× (2.45–2.52) | 0.85× (0.84–0.86) | 0.94× (0.93–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.33× (0.31–0.34) | 0.98× (0.98–0.99) | 0.79× (0.78–0.80) | 11.10× (11.06–11.14) | 0.27× (0.26–0.27) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.43× (0.39–0.49) | 0.70× (0.67–0.73) | 0.59× (0.57–0.61) | 5.60× (5.41–5.79) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.36× (0.34–0.38) | 0.55× (0.47–0.65) | 0.64× (0.60–0.68) | 0.55× (0.52–0.59) | 3.74× (3.55–3.94) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.03× (8.00–8.05) | 0.65× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.69–0.73) | 0.68× (0.64–0.72) | 1.70× (1.65–1.75) | 0.85× (0.82–0.88) | 6.16× (5.97–6.36) | 0.50× (0.49–0.52) | 0.53× (0.51–0.54) | 0.57× (0.56–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.67–0.72) | 0.72× (0.68–0.77) | 1.23× (1.18–1.28) | 0.76× (0.74–0.79) | 3.63× (3.50–3.75) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.18× (5.17–5.19) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.42) | 0.89× (0.86–0.93) | 0.71× (0.67–0.74) | 3.10× (2.99–3.20) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.82× (0.81–0.84) | 2.54× (2.49–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.22× (5.21–5.24) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.35× (0.32–0.38) | 0.84× (0.81–0.87) | 0.72× (0.69–0.76) | 2.89× (2.80–2.99) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.60–0.61) | 0.51× (0.49–0.54) | 0.80× (0.79–0.81) | 0.80× (0.76–0.84) | 2.31× (2.28–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.21–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.35× (5.33–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.89–0.91) | 6.22× (6.20–6.23) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.18–6.22) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.40) | 0.65× (0.64–0.66) | 6.41× (6.39–6.44) | 0.39× (0.38–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.30–0.31) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 34 MB | 0.50× (0.49–0.51) | 0.45× (0.41–0.49) | 1.12× (1.09–1.16) | 0.76× (0.74–0.78) | 3.67× (3.60–3.75) | 0.22× (0.22–0.23) | 0.24× (0.23–0.25) | 0.36× (0.36–0.37) | 0.18× (0.17–0.18) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 55 MB | 0.51× (0.50–0.52) | 0.45× (0.41–0.48) | 0.91× (0.89–0.94) | 0.74× (0.71–0.77) | 2.36× (2.31–2.42) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 549 | 1.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 986 | 1.96 | 1.07× (1.06–1.08) | 89% | 1.02× (1.02–1.03) | 1.04× (1.03–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,175 | 3.12 | 1.69× (1.68–1.71) | 54% | 1.20× (1.19–1.21) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (1,175 MB/s); lbzip2 at 4 threads (246 MB/s); pbz2 at 4 threads (369 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.79, mibz2 4t 1.05, lbzip2 1t 0.06, lbzip2 2t 0.21, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.50 | 0.01 | 0.08 | 0.178 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3696 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.83 | 9.9% |
| MTF + RLE2 | 1.58 | 18.9% |
| T-vector build | 1.48 | 17.8% |
| walk (the one each build uses) | 4.03 | 48.2% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.36 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.390 | 1.060 | (in its total) | 2.72× (2.71–2.73) |  |
| walk | 1.037 | 2.055 | (in its total) | 1.98× (1.97–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.038 | 0.040 | (in its total) | 1.04× (1.02–1.06) |  |
| inverse BWT, total | 1.465 | 3.155 | 1.876 | 2.15× (2.15–2.16) | 1.28× (1.28–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m45s ± 6m41s |
| 4 threads | 26m03s ± 3m40s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m16s ± 2m36s |
| the scanner alone (its serial part) | 8m09s ± 32.2s |
| the workers alone, over 4 | 18m16s ± 2m36s |

At 4 threads the workers bound it. Per byte: the scanner 4.7 ns a byte of bz2, the workers 2.39 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,220,170 | 14,849,844 | \`3f9b2018b96fd2e0…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
