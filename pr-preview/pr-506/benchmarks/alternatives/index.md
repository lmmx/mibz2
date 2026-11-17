# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-506/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`7084ec3`](<https://github.com/lmmx/mibz2/commit/7084ec36e868a59ffdd3f08089c8ea7e21886be4>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44048149928>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.517 s | 406 | 2.0 | 3.12× (3.04–3.19) | 6.91× (6.74–7.09) | 7.33× (7.17–7.50) | 8.21× (8.03–8.39) | 7.44× (7.25–7.64) | 7.98× (7.67–8.29) | 7.68× (7.47–7.90) | 2.43× (2.37–2.49) | 2.22× (2.16–2.28) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.279 s | 751 | 2.1 | 3.08× (2.90–3.27) | 6.39× (6.20–6.58) | 7.66× (7.43–7.90) | 7.80× (7.48–8.12) | 6.92× (6.67–7.19) | 14.09× (13.45–14.75) | 13.57× (13.02–14.14) | 4.29× (4.12–4.46) | 3.92× (3.76–4.09) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.230 s | 910 | 3.2 | 3.22× (3.16–3.29) | 5.62× (5.50–5.75) | 6.34× (6.15–6.54) | 6.43× (6.31–6.55) | 6.32× (6.19–6.46) | 17.27× (16.67–17.90) | 16.64× (16.19–17.09) | 5.26× (5.14–5.37) | 4.81× (4.70–4.92) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 990 | 0.9 | 0.57× (0.56–0.59) | 3.33× (3.21–3.45) | 4.35× (4.23–4.47) | 3.69× (3.59–3.79) | 6.10× (5.96–6.25) | 2.72× (2.64–2.80) | 2.70× (2.62–2.78) | 1.11× (1.09–1.13) | 1.01× (0.97–1.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 980 | 0.9 | 0.55× (0.53–0.56) | 2.89× (2.81–2.97) | 5.84× (5.69–6.00) | 3.78× (3.68–3.88) | 6.13× (5.99–6.28) | 2.77× (2.69–2.84) | 2.75× (2.67–2.82) | 1.13× (1.09–1.17) | 1.03× (0.99–1.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 976 | 0.9 | 0.56× (0.54–0.58) | 2.86× (2.67–3.07) | 5.72× (5.36–6.11) | 3.67× (3.45–3.91) | 6.04× (5.66–6.46) | 2.69× (2.50–2.88) | 2.67× (2.49–2.85) | 1.09× (1.02–1.17) | 1.00× (0.93–1.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 511 | 1.6 | 2.81× (2.75–2.87) | 3.27× (3.19–3.35) | 3.85× (3.77–3.94) | 4.31× (4.23–4.40) | 4.79× (4.72–4.86) | 3.54× (3.49–3.58) | 2.88× (2.84–2.93) | 2.07× (2.04–2.11) | 2.10× (2.05–2.14) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 792 | 1.9 | 2.30× (2.19–2.42) | 2.92× (2.78–3.07) | 4.00× (3.81–4.20) | 3.83× (3.64–4.03) | 5.16× (4.98–5.34) | 5.45× (5.23–5.68) | 4.45× (4.28–4.62) | 3.19× (3.05–3.34) | 3.23× (3.11–3.36) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 845 | 3.3 | 2.14× (2.06–2.23) | 2.14× (2.00–2.29) | 3.08× (2.93–3.24) | 2.53× (2.41–2.65) | 4.41× (4.23–4.61) | 5.73× (5.48–5.99) | 4.67× (4.48–4.88) | 3.36× (3.23–3.49) | 3.40× (3.25–3.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.146 s | 55 | 14.8 | 1.33× (1.32–1.34) | 2.71× (2.66–2.76) | 2.42× (2.37–2.47) | 2.52× (2.49–2.56) | 2.80× (2.75–2.85) | 3.13× (3.10–3.17) | 3.30× (3.25–3.34) | 1.27× (1.26–1.28) | 1.26× (1.24–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.088 s | 90 | 16.5 | 1.25× (1.23–1.27) | 2.58× (2.48–2.68) | 2.41× (2.35–2.46) | 3.08× (3.04–3.13) | 2.81× (2.75–2.86) | 5.13× (5.07–5.18) | 5.39× (5.29–5.50) | 2.08× (2.04–2.12) | 2.06× (2.02–2.10) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 113 | 21.4 | 1.36× (1.33–1.39) | 2.45× (2.34–2.56) | 2.30× (2.24–2.37) | 2.92× (2.85–2.99) | 2.90× (2.83–2.98) | 6.33× (6.17–6.49) | 6.66× (6.49–6.83) | 2.57× (2.50–2.64) | 2.54× (2.47–2.62) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.162 s | 59 | 13.6 | 1.38× (1.37–1.39) | 3.13× (3.09–3.18) | 2.61× (2.56–2.66) | 2.71× (2.66–2.76) | 3.03× (2.99–3.08) | 3.84× (3.80–3.88) | 4.10× (4.07–4.14) | 1.50× (1.49–1.51) | 1.51× (1.49–1.53) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.095 s | 101 | 14.9 | 1.30× (1.27–1.33) | 2.88× (2.80–2.97) | 2.55× (2.49–2.61) | 3.61× (3.46–3.76) | 3.04× (2.97–3.12) | 6.48× (6.34–6.62) | 6.93× (6.79–7.07) | 2.53× (2.48–2.59) | 2.55× (2.49–2.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 20.7 | 1.46× (1.40–1.52) | 2.83× (2.71–2.95) | 2.49× (2.42–2.57) | 3.31× (3.20–3.42) | 3.00× (2.90–3.11) | 7.86× (7.61–8.12) | 8.40× (8.17–8.65) | 3.07× (2.98–3.16) | 3.09× (3.00–3.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 749 | 1.2 | 0.63× (0.61–0.65) | 3.12× (2.93–3.32) | 4.20× (4.03–4.39) | 3.20× (3.09–3.31) | 6.71× (6.50–6.92) | 2.35× (2.27–2.45) | 2.41× (2.34–2.49) | 1.05× (1.02–1.08) | 1.26× (1.02–1.55) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 722 | 1.3 | 0.52× (0.51–0.54) | 2.70× (2.51–2.91) | 4.88× (4.74–5.03) | 3.14× (3.04–3.25) | 6.52× (6.33–6.72) | 2.27× (2.20–2.33) | 2.33× (2.27–2.39) | 1.01× (0.97–1.05) | 1.21× (0.99–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 1.3 | 0.55× (0.51–0.60) | 2.78× (2.63–2.93) | 4.86× (4.79–4.93) | 3.12× (3.07–3.18) | 6.52× (6.18–6.87) | 2.26× (2.21–2.31) | 2.31× (2.27–2.36) | 1.00× (0.98–1.02) | 1.20× (1.00–1.46) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 0.071 s | 171 | 4.8 | 1.79× (1.76–1.81) | 4.00× (3.88–4.12) | 4.38× (4.27–4.49) | 4.59× (4.52–4.66) | 4.70× (4.60–4.80) | 4.92× (4.83–5.02) | 4.94× (4.85–5.03) | 1.49× (1.45–1.52) | 1.45× (1.42–1.49) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 0.046 s | 266 | 5.5 | 1.52× (1.48–1.57) | 3.39× (3.25–3.54) | 4.01× (3.90–4.12) | 4.63× (4.49–4.77) | 4.39× (4.32–4.46) | 7.49× (7.31–7.68) | 7.52× (7.33–7.71) | 2.26× (2.21–2.32) | 2.21× (2.17–2.26) | yes |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 0.040 s | 308 | 8.2 | 1.53× (1.48–1.57) | 2.72× (2.64–2.81) | 3.20× (3.13–3.26) | 3.96× (3.84–4.10) | 3.99× (3.88–4.10) | 8.48× (8.26–8.71) | 8.51× (8.28–8.74) | 2.56× (2.49–2.63) | 2.50× (2.40–2.61) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.11× (3.03–3.19) | 6.88× (6.71–7.06) | 7.30× (7.14–7.46) | 8.18× (8.00–8.36) | 7.42× (7.23–7.62) | 7.95× (7.65–8.26) | 7.66× (7.45–7.87) | 2.42× (2.36–2.48) | 2.20× (2.15–2.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.98× (2.87–3.10) | 6.58× (6.41–6.76) | 7.23× (7.04–7.42) | 8.10× (7.89–8.31) | 6.90× (6.71–7.09) | 7.39× (7.09–7.69) | 7.12× (6.89–7.35) | 2.25× (2.18–2.32) | 2.05× (1.98–2.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.22× (3.10–3.35) | 6.15× (5.89–6.41) | 6.29× (6.03–6.57) | 7.03× (6.75–7.32) | 6.23× (5.94–6.52) | 4.82× (4.60–5.04) | 4.64× (4.45–4.84) | 1.47× (1.41–1.53) | 1.34× (1.28–1.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.62) | 2.80× (2.71–2.90) | 3.84× (3.75–3.94) | 3.49× (3.40–3.58) | 5.72× (5.59–5.85) | 2.59× (2.52–2.67) | 2.57× (2.50–2.65) | 1.10× (1.08–1.12) | 0.89× (0.85–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.60) | 2.83× (2.77–2.90) | 5.11× (4.99–5.23) | 3.54× (3.46–3.62) | 5.68× (5.56–5.80) | 2.60× (2.54–2.66) | 2.58× (2.52–2.65) | 1.10× (1.07–1.14) | 0.89× (0.87–0.91) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.57–0.60) | 3.21× (3.03–3.40) | 5.03× (4.73–5.35) | 3.49× (3.28–3.72) | 5.65× (5.30–6.03) | 2.56× (2.39–2.74) | 2.54× (2.38–2.71) | 1.09× (1.01–1.16) | 0.87× (0.82–0.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.67–2.77) | 3.07× (3.02–3.12) | 3.60× (3.54–3.65) | 4.17× (4.09–4.25) | 4.63× (4.56–4.69) | 3.43× (3.38–3.47) | 2.80× (2.76–2.85) | 2.02× (1.99–2.06) | 1.92× (1.89–1.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.38–2.50) | 2.78× (2.71–2.86) | 3.27× (3.19–3.36) | 3.64× (3.54–3.75) | 3.98× (3.87–4.10) | 2.92× (2.84–3.00) | 2.38× (2.32–2.44) | 1.72× (1.68–1.76) | 1.63× (1.58–1.68) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.27× (2.21–2.33) | 1.94× (1.89–2.00) | 2.38× (2.30–2.45) | 2.37× (2.28–2.45) | 2.45× (2.39–2.51) | 1.65× (1.61–1.70) | 1.35× (1.31–1.39) | 0.97× (0.95–1.00) | 0.92× (0.89–0.95) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.34) | 2.66× (2.62–2.70) | 2.37× (2.32–2.41) | 2.51× (2.48–2.54) | 2.78× (2.73–2.83) | 3.11× (3.08–3.14) | 3.27× (3.23–3.32) | 1.27× (1.26–1.28) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.27) | 2.60× (2.48–2.72) | 2.23× (2.19–2.28) | 2.97× (2.93–3.02) | 2.54× (2.49–2.58) | 2.81× (2.76–2.86) | 2.95× (2.89–3.02) | 1.15× (1.13–1.17) | 1.11× (1.10–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.42–1.57) | 2.62× (2.54–2.71) | 2.19× (2.14–2.24) | 2.90× (2.81–2.99) | 2.56× (2.49–2.63) | 2.09× (2.04–2.15) | 2.20× (2.15–2.26) | 0.85× (0.83–0.87) | 0.83× (0.81–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.38× (1.36–1.39) | 3.08× (3.04–3.13) | 2.54× (2.50–2.59) | 2.69× (2.65–2.75) | 3.01× (2.97–3.06) | 3.81× (3.77–3.85) | 4.07× (4.04–4.11) | 1.50× (1.48–1.51) | 1.48× (1.47–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.31× (1.27–1.36) | 2.90× (2.84–2.97) | 2.40× (2.35–2.45) | 3.34× (3.25–3.42) | 2.79× (2.72–2.85) | 3.44× (3.36–3.52) | 3.67× (3.59–3.76) | 1.35× (1.31–1.39) | 1.34× (1.30–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.58–1.64) | 3.02× (2.94–3.10) | 2.24× (2.20–2.29) | 3.11× (3.03–3.19) | 2.71× (2.65–2.77) | 2.49× (2.44–2.53) | 2.66× (2.61–2.71) | 0.98× (0.96–0.99) | 0.97× (0.95–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.65–0.69) | 2.44× (2.32–2.56) | 3.42× (3.31–3.54) | 2.97× (2.86–3.07) | 6.11× (5.91–6.31) | 2.21× (2.13–2.30) | 2.27× (2.20–2.34) | 1.05× (1.02–1.08) | 0.83× (0.80–0.86) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.56–0.60) | 2.26× (2.20–2.32) | 4.16× (4.06–4.26) | 2.92× (2.84–3.01) | 5.86× (5.70–6.03) | 2.09× (2.04–2.15) | 2.15× (2.10–2.20) | 0.99× (0.96–1.03) | 0.78× (0.76–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.55–0.66) | 2.68× (2.61–2.74) | 4.17× (4.11–4.24) | 2.98× (2.92–3.03) | 5.98× (5.67–6.32) | 2.13× (2.09–2.18) | 2.19× (2.15–2.23) | 1.01× (0.99–1.03) | 0.80× (0.79–0.81) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 1.77× (1.74–1.79) | 3.85× (3.73–3.98) | 4.22× (4.14–4.31) | 4.50× (4.44–4.57) | 4.61× (4.52–4.70) | 4.83× (4.74–4.92) | 4.84× (4.76–4.93) | 1.47× (1.44–1.51) | 1.39× (1.37–1.40) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 1.62× (1.57–1.66) | 3.42× (3.35–3.49) | 3.93× (3.89–3.97) | 4.31× (4.28–4.34) | 4.02× (3.95–4.09) | 4.12× (4.05–4.18) | 4.13× (4.07–4.18) | 1.26× (1.24–1.28) | 1.18× (1.17–1.19) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 1.86× (1.81–1.91) | 3.14× (3.07–3.21) | 3.47× (3.37–3.57) | 3.77× (3.66–3.88) | 3.60× (3.53–3.66) | 2.76× (2.70–2.82) | 2.76× (2.72–2.81) | 0.84× (0.82–0.86) | 0.79× (0.77–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.79) | 11.09× (11.05–11.12) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.48× (0.42–0.54) | 0.72× (0.70–0.74) | 0.61× (0.59–0.63) | 5.77× (5.61–5.93) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.33–0.36) | 0.50× (0.44–0.56) | 0.60× (0.58–0.62) | 0.54× (0.51–0.56) | 3.55× (3.42–3.69) | 0.08× (0.08–0.09) | 0.09× (0.09–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.42–0.45) | 1.70× (1.69–1.71) | 0.66× (0.65–0.66) | 8.04× (8.00–8.07) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.69× (0.64–0.75) | 1.61× (1.58–1.63) | 0.81× (0.80–0.82) | 5.88× (5.80–5.97) | 0.48× (0.47–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.66) | 0.69× (0.62–0.77) | 1.13× (1.09–1.17) | 0.70× (0.67–0.73) | 3.40× (3.27–3.54) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.21× (5.19–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.39× (0.36–0.42) | 0.90× (0.87–0.92) | 0.68× (0.65–0.71) | 3.11× (3.03–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.61–0.66) | 0.45× (0.44–0.46) | 0.85× (0.83–0.87) | 0.80× (0.78–0.82) | 2.49× (2.43–2.55) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.23× (5.21–5.24) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.50) | 0.38× (0.36–0.40) | 0.86× (0.82–0.89) | 0.70× (0.67–0.74) | 2.94× (2.84–3.06) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.62) | 0.49× (0.47–0.51) | 0.81× (0.80–0.83) | 0.83× (0.79–0.86) | 2.35× (2.31–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.23) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.37–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.28–5.32) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.77) | 5.31× (5.30–5.33) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.66× (0.66–0.66) | 6.42× (6.39–6.44) | 0.40× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.32) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 2 | 38 MB | 0.44× (0.44–0.45) | 0.40× (0.37–0.43) | 1.02× (1.00–1.05) | 0.68× (0.66–0.69) | 3.26× (3.21–3.32) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.41 MB → 12.2 MB, 5.1×) | 4 | 59 MB | 0.48× (0.47–0.49) | 0.45× (0.43–0.48) | 0.87× (0.85–0.89) | 0.69× (0.66–0.73) | 2.23× (2.19–2.28) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 406 | 2.47 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 751 | 2.62 | 1.08× (1.06–1.09) | 88% | 1.03× (1.02–1.04) | 1.03× (1.01–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 910 | 4.06 | 1.65× (1.60–1.70) | 54% | 1.48× (1.46–1.49) | 1.71× (1.69–1.73) |

**Best:** mibz2 at 4 threads (910 MB/s); lbzip2 at 4 threads (159 MB/s); pbz2 at 4 threads (279 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.18, lbzip2 1t 0.04, lbzip2 2t 0.19, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.64 | 0.02 | 0.08 | 0.230 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 59.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.26 | 11.0% |
| MTF + RLE2 | 0.37 | 15.8% |
| T-vector build | 0.36 | 15.4% |
| walk (the one each build uses) | 1.26 | 54.0% |
| of which: segment assembly | 0.06 | 2.7% |
| RLE1 | 0.06 | 2.5% |
| CRC | 0.03 | 1.3% |
| **total** | 2.33 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.438 | 1.141 | (in its total) | 2.60× (2.59–2.62) |  |
| walk | 1.517 | 3.004 | (in its total) | 1.98× (1.97–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.074 | (in its total) | 0.92× (0.91–0.93) |  |
| inverse BWT, total | 2.033 | 4.219 | 2.132 | 2.08× (2.06–2.09) | 1.05× (1.04–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h12m ± 9m16s |
| 4 threads | 33m13s ± 4m56s |
| 4 threads, the sample's overlap gap as a fixed cost | 22m48s ± 3m41s |
| the scanner alone (its serial part) | 13m41s ± 52.7s |
| the workers alone, over 4 | 22m48s ± 3m41s |

At 4 threads the workers bound it. Per byte: the scanner 7.9 ns a byte of bz2, the workers 2.99 ns a byte of output at 4 threads (CPU, summed over them), 1.91 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,408,661 | 12,232,367 | \`2d4782182c8b0988…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
