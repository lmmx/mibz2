# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-631/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`6fc3b87`](<https://github.com/lmmx/mibz2/commit/6fc3b87ad12ea208cc8a718cdb945e5492f88258>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44527977717>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.521 s | 402 | 7.5 | 3.09× (3.06–3.11) | 6.91× (6.83–6.99) | 7.38× (7.34–7.42) | 8.28× (8.23–8.34) | 7.39× (7.34–7.44) | 7.93× (7.87–7.99) | 7.54× (7.50–7.59) | 2.38× (2.36–2.39) | 2.21× (2.20–2.22) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.302 s | 694 | 8.1 | 3.08× (2.96–3.20) | 6.31× (6.09–6.54) | 7.59× (7.41–7.78) | 7.81× (7.65–7.97) | 6.90× (6.74–7.06) | 13.71× (13.31–14.13) | 13.05× (12.69–13.42) | 4.11× (3.99–4.23) | 3.82× (3.71–3.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.245 s | 854 | 12.9 | 3.31× (3.24–3.38) | 5.71× (5.57–5.86) | 6.52× (6.36–6.69) | 6.63× (6.53–6.74) | 6.36× (6.26–6.46) | 16.82× (16.68–16.96) | 16.01× (15.86–16.17) | 5.04× (5.00–5.08) | 4.68× (4.65–4.71) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 941 | 3.5 | 0.58× (0.57–0.59) | 3.37× (3.32–3.42) | 4.32× (4.17–4.46) | 3.81× (3.74–3.87) | 6.25× (6.15–6.35) | 2.84× (2.79–2.89) | 2.83× (2.79–2.87) | 1.17× (1.14–1.20) | 0.98× (0.96–0.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 937 | 3.5 | 0.54× (0.53–0.55) | 2.88× (2.84–2.91) | 5.72× (5.60–5.85) | 3.79× (3.75–3.83) | 6.13× (6.05–6.20) | 2.82× (2.79–2.84) | 2.81× (2.78–2.84) | 1.16× (1.14–1.18) | 0.97× (0.96–0.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 924 | 3.5 | 0.54× (0.54–0.55) | 2.84× (2.81–2.88) | 5.61× (5.54–5.68) | 3.73× (3.68–3.77) | 6.07× (6.00–6.14) | 2.78× (2.75–2.81) | 2.77× (2.74–2.80) | 1.14× (1.12–1.17) | 0.96× (0.94–0.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.039 s | 501 | 6.3 | 2.79× (2.76–2.83) | 3.38× (3.35–3.42) | 3.86× (3.82–3.91) | 4.42× (4.38–4.47) | 4.78× (4.74–4.83) | 3.64× (3.60–3.68) | 2.93× (2.90–2.96) | 2.11× (2.08–2.15) | 2.08× (2.06–2.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 803 | 7.4 | 2.36× (2.26–2.46) | 2.92× (2.76–3.09) | 4.10× (3.98–4.23) | 3.86× (3.69–4.03) | 5.16× (4.97–5.37) | 5.64× (5.39–5.90) | 4.54× (4.34–4.75) | 3.27× (3.13–3.42) | 3.22× (3.08–3.37) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 741 | 14.1 | 1.99× (1.90–2.08) | 1.98× (1.90–2.06) | 2.88× (2.68–3.09) | 2.42× (2.25–2.61) | 4.02× (3.87–4.18) | 5.33× (5.13–5.54) | 4.29× (4.13–4.46) | 3.09× (2.97–3.22) | 3.04× (2.93–3.16) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.158 s | 51 | 60.0 | 1.28× (1.27–1.30) | 2.55× (2.52–2.59) | 2.23× (2.20–2.26) | 2.30× (2.25–2.35) | 2.60× (2.56–2.65) | 2.94× (2.90–2.99) | 3.03× (2.99–3.07) | 1.27× (1.25–1.29) | 1.28× (1.26–1.31) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.093 s | 86 | 65.2 | 1.24× (1.20–1.27) | 2.47× (2.40–2.55) | 2.34× (2.27–2.42) | 3.03× (2.90–3.16) | 2.81× (2.71–2.92) | 5.00× (4.85–5.16) | 5.15× (5.00–5.30) | 2.16× (2.11–2.21) | 2.18× (2.13–2.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.076 s | 105 | 88.3 | 1.37× (1.34–1.40) | 2.34× (2.27–2.41) | 2.29× (2.23–2.35) | 2.80× (2.76–2.85) | 2.87× (2.80–2.94) | 6.12× (5.98–6.25) | 6.30× (6.16–6.44) | 2.64× (2.56–2.72) | 2.67× (2.59–2.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.175 s | 55 | 55.2 | 1.35× (1.33–1.37) | 2.98× (2.93–3.05) | 2.42× (2.38–2.47) | 2.51× (2.47–2.56) | 2.84× (2.79–2.89) | 3.60× (3.53–3.67) | 3.84× (3.76–3.91) | 1.49× (1.47–1.51) | 1.51× (1.48–1.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.105 s | 91 | 60.3 | 1.23× (1.18–1.28) | 2.71× (2.58–2.85) | 2.45× (2.27–2.66) | 3.39× (3.25–3.54) | 2.90× (2.79–3.01) | 6.05× (5.83–6.28) | 6.45× (6.22–6.68) | 2.50× (2.39–2.62) | 2.54× (2.42–2.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.086 s | 112 | 84.8 | 1.41× (1.37–1.46) | 2.60× (2.51–2.70) | 2.36× (2.27–2.46) | 3.21× (3.10–3.32) | 2.84× (2.76–2.92) | 7.34× (7.11–7.59) | 7.83× (7.61–8.07) | 3.04× (2.92–3.16) | 3.09× (2.96–3.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 717 | 4.7 | 0.63× (0.62–0.64) | 2.94× (2.90–2.99) | 4.32× (4.25–4.38) | 3.28× (3.22–3.34) | 6.78× (6.67–6.88) | 2.45× (2.41–2.49) | 2.45× (2.42–2.49) | 1.05× (1.04–1.07) | 1.48× (1.46–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 687 | 5.0 | 0.54× (0.53–0.55) | 2.83× (2.79–2.88) | 4.93× (4.77–5.10) | 3.17× (3.12–3.22) | 6.53× (6.43–6.63) | 2.35× (2.30–2.40) | 2.36× (2.32–2.40) | 1.01× (0.99–1.03) | 1.43× (1.40–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 4.9 | 0.53× (0.53–0.54) | 2.80× (2.77–2.82) | 4.92× (4.71–5.14) | 3.13× (3.10–3.16) | 6.40× (6.32–6.48) | 2.32× (2.29–2.34) | 2.32× (2.30–2.35) | 1.00× (0.98–1.01) | 1.40× (1.39–1.42) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.077 s | 170 | 18.1 | 1.81× (1.79–1.82) | 3.85× (3.81–3.90) | 4.36× (4.32–4.40) | 4.50× (4.46–4.54) | 4.57× (4.52–4.61) | 4.89× (4.84–4.93) | 4.77× (4.70–4.84) | 1.47× (1.46–1.48) | 1.44× (1.43–1.46) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.049 s | 266 | 21.0 | 1.58× (1.50–1.67) | 3.48× (3.11–3.89) | 4.29× (4.04–4.56) | 4.60× (4.31–4.92) | 4.40× (4.19–4.61) | 7.44× (7.10–7.79) | 7.26× (6.91–7.63) | 2.24× (2.13–2.35) | 2.20× (2.10–2.30) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.042 s | 313 | 31.8 | 1.72× (1.65–1.79) | 3.02× (2.92–3.13) | 3.95× (3.73–4.18) | 4.13× (3.94–4.32) | 4.39× (4.27–4.51) | 8.95× (8.75–9.15) | 8.74× (8.55–8.94) | 2.69× (2.64–2.75) | 2.64× (2.59–2.70) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.08× (3.06–3.10) | 6.88× (6.80–6.96) | 7.35× (7.32–7.39) | 8.26× (8.20–8.31) | 7.37× (7.32–7.42) | 7.90× (7.84–7.96) | 7.52× (7.47–7.57) | 2.37× (2.35–2.39) | 2.19× (2.18–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.96× (2.92–3.01) | 6.55× (6.46–6.65) | 7.27× (7.22–7.32) | 8.13× (8.05–8.21) | 6.93× (6.85–7.01) | 7.24× (7.18–7.30) | 6.89× (6.85–6.93) | 2.17× (2.15–2.19) | 2.01× (2.00–2.02) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.27× (3.21–3.32) | 6.13× (6.01–6.25) | 6.37× (6.30–6.45) | 7.12× (7.01–7.23) | 6.21× (6.09–6.33) | 4.61× (4.53–4.68) | 4.39× (4.33–4.45) | 1.38× (1.36–1.40) | 1.28× (1.26–1.29) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.87× (2.82–2.93) | 3.97× (3.91–4.03) | 3.58× (3.53–3.64) | 5.83× (5.75–5.92) | 2.69× (2.65–2.73) | 2.68× (2.65–2.72) | 1.15× (1.12–1.18) | 0.91× (0.89–0.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.85× (2.82–2.88) | 5.18× (5.12–5.23) | 3.55× (3.51–3.58) | 5.68× (5.62–5.74) | 2.64× (2.62–2.67) | 2.64× (2.61–2.66) | 1.13× (1.11–1.15) | 0.90× (0.88–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.58) | 3.25× (3.21–3.30) | 5.13× (5.07–5.19) | 3.54× (3.51–3.58) | 5.67× (5.61–5.73) | 2.64× (2.62–2.67) | 2.63× (2.61–2.66) | 1.13× (1.11–1.15) | 0.90× (0.88–0.91) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.68–2.75) | 3.12× (3.09–3.15) | 3.67× (3.65–3.70) | 4.28× (4.24–4.31) | 4.62× (4.58–4.66) | 3.53× (3.49–3.56) | 2.85× (2.82–2.88) | 2.07× (2.03–2.10) | 1.95× (1.92–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.45× (2.40–2.50) | 2.81× (2.73–2.89) | 3.31× (3.25–3.38) | 3.66× (3.58–3.75) | 3.95× (3.87–4.04) | 2.95× (2.87–3.02) | 2.38× (2.32–2.44) | 1.73× (1.68–1.77) | 1.63× (1.58–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.15× (2.06–2.24) | 1.80× (1.72–1.88) | 2.23× (2.13–2.34) | 2.20× (2.10–2.32) | 2.27× (2.17–2.37) | 1.55× (1.49–1.61) | 1.25× (1.20–1.30) | 0.91× (0.88–0.94) | 0.85× (0.82–0.89) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.28× (1.26–1.30) | 2.50× (2.47–2.53) | 2.19× (2.17–2.21) | 2.28× (2.23–2.34) | 2.58× (2.54–2.63) | 2.92× (2.88–2.96) | 3.01× (2.97–3.04) | 1.27× (1.25–1.28) | 1.25× (1.23–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.22–1.24) | 2.46× (2.43–2.49) | 2.14× (2.11–2.17) | 2.85× (2.82–2.87) | 2.48× (2.45–2.52) | 2.70× (2.67–2.73) | 2.78× (2.75–2.81) | 1.17× (1.15–1.19) | 1.15× (1.13–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.40–1.54) | 2.53× (2.49–2.57) | 2.13× (2.11–2.16) | 2.76× (2.71–2.80) | 2.50× (2.45–2.54) | 1.99× (1.97–2.00) | 2.04× (2.02–2.07) | 0.86× (0.84–0.88) | 0.85× (0.83–0.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.34× (1.32–1.36) | 2.94× (2.88–2.99) | 2.38× (2.34–2.41) | 2.50× (2.45–2.55) | 2.82× (2.77–2.87) | 3.57× (3.51–3.64) | 3.81× (3.74–3.88) | 1.48× (1.47–1.50) | 1.48× (1.46–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.26× (1.22–1.30) | 2.77× (2.69–2.85) | 2.33× (2.19–2.49) | 3.19× (3.08–3.30) | 2.65× (2.57–2.74) | 3.23× (3.13–3.33) | 3.44× (3.34–3.55) | 1.34× (1.29–1.39) | 1.34× (1.29–1.40) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.53–1.61) | 2.88× (2.82–2.94) | 2.15× (2.12–2.19) | 3.04× (2.96–3.12) | 2.59× (2.55–2.64) | 2.33× (2.29–2.38) | 2.49× (2.44–2.54) | 0.97× (0.95–0.99) | 0.97× (0.94–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.68) | 2.43× (2.40–2.46) | 3.48× (3.43–3.53) | 3.03× (2.98–3.09) | 6.15× (6.06–6.23) | 2.29× (2.25–2.33) | 2.30× (2.26–2.33) | 1.05× (1.03–1.07) | 0.83× (0.82–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.30× (2.26–2.35) | 4.27× (4.20–4.33) | 2.95× (2.90–2.99) | 5.85× (5.76–5.93) | 2.16× (2.12–2.20) | 2.17× (2.13–2.20) | 0.99× (0.97–1.01) | 0.79× (0.77–0.81) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.63× (2.58–2.67) | 4.26× (4.18–4.34) | 2.99× (2.95–3.02) | 5.87× (5.79–5.96) | 2.18× (2.16–2.21) | 2.19× (2.16–2.21) | 1.00× (0.99–1.02) | 0.80× (0.78–0.81) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.79× (1.77–1.80) | 3.74× (3.71–3.77) | 4.24× (4.19–4.28) | 4.42× (4.38–4.46) | 4.48× (4.44–4.53) | 4.80× (4.76–4.84) | 4.68× (4.62–4.75) | 1.46× (1.45–1.47) | 1.38× (1.37–1.39) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.63× (1.62–1.65) | 3.45× (3.39–3.51) | 4.01× (3.96–4.05) | 4.47× (4.40–4.55) | 3.99× (3.97–4.01) | 4.13× (4.09–4.16) | 4.03× (3.97–4.08) | 1.26× (1.24–1.27) | 1.19× (1.18–1.20) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.84× (1.78–1.91) | 3.15× (3.09–3.20) | 3.52× (3.46–3.58) | 3.88× (3.78–3.97) | 3.53× (3.50–3.56) | 2.73× (2.70–2.77) | 2.67× (2.63–2.70) | 0.83× (0.82–0.84) | 0.79× (0.78–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.26× (0.26–0.26) | 0.97× (0.97–0.98) | 0.78× (0.78–0.78) | 11.01× (10.98–11.03) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.42× (0.42–0.42) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.35× (0.31–0.40) | 0.67× (0.65–0.70) | 0.58× (0.56–0.60) | 5.42× (5.22–5.63) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.32–0.34) | 0.45× (0.39–0.52) | 0.57× (0.56–0.59) | 0.51× (0.49–0.53) | 3.42× (3.32–3.53) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.36–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.03× (8.02–8.05) | 0.65× (0.64–0.66) | 0.69× (0.68–0.70) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.64× (0.61–0.67) | 1.59× (1.55–1.64) | 0.81× (0.79–0.83) | 5.84× (5.69–5.99) | 0.47× (0.46–0.48) | 0.50× (0.48–0.51) | 0.54× (0.52–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.67× (0.61–0.73) | 1.14× (1.10–1.17) | 0.70× (0.68–0.72) | 3.41× (3.32–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.42× (0.41–0.43) | 0.90× (0.89–0.92) | 0.69× (0.66–0.73) | 3.14× (3.07–3.21) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.62–0.65) | 0.45× (0.43–0.46) | 0.85× (0.83–0.87) | 0.80× (0.78–0.82) | 2.48× (2.43–2.54) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.21) | 0.32× (0.32–0.33) | 0.34× (0.34–0.34) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.50) | 0.37× (0.34–0.41) | 0.87× (0.83–0.90) | 0.70× (0.66–0.75) | 2.96× (2.85–3.07) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.46× (0.44–0.49) | 0.82× (0.81–0.83) | 0.85× (0.82–0.87) | 2.36× (2.33–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.31× (5.30–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.43) | 0.67× (0.66–0.67) | 6.45× (6.44–6.47) | 0.40× (0.40–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 37 MB | 0.45× (0.45–0.46) | 0.40× (0.36–0.44) | 1.05× (1.03–1.07) | 0.66× (0.63–0.70) | 3.36× (3.31–3.40) | 0.21× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 56 MB | 0.49× (0.48–0.51) | 0.47× (0.43–0.52) | 0.90× (0.87–0.92) | 0.71× (0.68–0.75) | 2.29× (2.24–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 402 | 2.49 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 694 | 2.71 | 1.09× (1.09–1.10) | 86% | 1.04× (1.03–1.05) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 854 | 4.31 | 1.72× (1.69–1.74) | 53% | 1.53× (1.49–1.56) | 1.82× (1.79–1.85) |

**Best:** mibz2 at 4 threads (854 MB/s); lbzip2 at 4 threads (150 MB/s); pbz2 at 4 threads (259 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.18, lbzip2 1t 0.05, lbzip2 2t 0.16, lbzip2 4t 0.27, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.34. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.69 | 0.02 | 0.09 | 0.245 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3000 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.17 | 11.5% |
| MTF + RLE2 | 1.68 | 16.5% |
| T-vector build | 1.58 | 15.6% |
| walk (the one each build uses) | 5.32 | 52.4% |
| of which: segment assembly | 0.28 | 2.8% |
| RLE1 | 0.27 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.505 | 1.318 | (in its total) | 2.61× (2.61–2.61) |  |
| walk | 1.638 | 3.082 | (in its total) | 1.88× (1.88–1.89) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.092 | 0.081 | (in its total) | 0.88× (0.87–0.89) |  |
| inverse BWT, total | 2.234 | 4.480 | 2.370 | 2.01× (2.00–2.01) | 1.06× (1.06–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m35s |
| 4 threads | 35m48s ± 5m01s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m04s ± 3m32s |
| the scanner alone (its serial part) | 14m50s ± 26.3s |
| the workers alone, over 4 | 25m04s ± 3m32s |

At 4 threads the workers bound it. Per byte: the scanner 8.6 ns a byte of bz2, the workers 3.29 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3000.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
