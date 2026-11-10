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

The latest record: [`ceb0b30`](<https://github.com/lmmx/mibz2/commit/ceb0b30b631d6253d48c40ab4bcb9e6d5dddab8c>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42849694118>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.606 s | 346 | 2.3 | 2.49× (2.42–2.56) | 5.57× (5.42–5.71) | 5.92× (5.77–6.07) | 6.68× (6.51–6.85) | 5.97× (5.80–6.14) | 6.41× (6.23–6.60) | 6.18× (6.03–6.32) | 1.93× (1.89–1.98) | 1.78× (1.74–1.83) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.334 s | 628 | 2.4 | 2.56× (2.48–2.63) | 5.41× (5.27–5.56) | 6.41× (6.20–6.62) | 6.60× (6.38–6.82) | 5.89× (5.67–6.12) | 11.61× (11.23–12.01) | 11.19× (10.78–11.62) | 3.51× (3.38–3.64) | 3.23× (3.12–3.35) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.282 s | 743 | 4.0 | 2.65× (2.59–2.70) | 4.59× (4.51–4.68) | 5.23× (5.12–5.33) | 5.27× (5.16–5.39) | 5.13× (5.06–5.20) | 13.91× (13.57–14.27) | 13.41× (13.11–13.71) | 4.20× (4.12–4.28) | 3.87× (3.79–3.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,002 | 0.9 | 0.58× (0.57–0.59) | 3.25× (3.08–3.44) | 4.50× (4.37–4.64) | 3.75× (3.66–3.83) | 6.24× (6.11–6.36) | 2.80× (2.75–2.85) | 2.82× (2.78–2.86) | 1.14× (1.12–1.16) | 1.03× (1.01–1.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 980 | 0.9 | 0.55× (0.53–0.57) | 3.01× (2.94–3.08) | 5.74× (5.56–5.92) | 3.67× (3.60–3.75) | 6.06× (5.96–6.17) | 2.75× (2.68–2.81) | 2.77× (2.72–2.82) | 1.12× (1.08–1.15) | 1.01× (0.99–1.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 980 | 0.9 | 0.54× (0.53–0.56) | 3.00× (2.92–3.08) | 5.74× (5.54–5.95) | 3.68× (3.61–3.75) | 6.16× (6.04–6.29) | 2.73× (2.67–2.79) | 2.75× (2.70–2.80) | 1.11× (1.08–1.14) | 1.01× (0.98–1.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 533 | 1.6 | 2.79× (2.76–2.83) | 3.30× (3.21–3.40) | 3.89× (3.72–4.07) | 4.34× (4.25–4.44) | 4.77× (4.66–4.88) | 3.63× (3.52–3.74) | 2.90× (2.83–2.98) | 2.07× (2.03–2.11) | 2.12× (2.04–2.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 875 | 1.8 | 2.51× (2.43–2.60) | 3.21× (3.05–3.37) | 4.18× (4.01–4.37) | 4.11× (3.98–4.25) | 5.57× (5.25–5.91) | 5.90× (5.67–6.14) | 4.73× (4.57–4.88) | 3.37× (3.27–3.49) | 3.44× (3.32–3.58) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.023 s | 852 | 3.3 | 2.13× (2.03–2.24) | 2.10× (1.91–2.31) | 3.13× (3.02–3.26) | 2.54× (2.36–2.73) | 4.38× (4.20–4.57) | 5.77× (5.57–5.97) | 4.62× (4.46–4.79) | 3.30× (3.18–3.43) | 3.37× (3.20–3.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.141 s | 57 | 14.2 | 1.37× (1.34–1.41) | 2.74× (2.71–2.78) | 2.48× (2.42–2.54) | 2.55× (2.50–2.59) | 2.80× (2.77–2.83) | 3.14× (3.08–3.20) | 3.28× (3.20–3.36) | 1.27× (1.25–1.29) | 1.28× (1.27–1.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.085 s | 95 | 15.7 | 1.26× (1.23–1.29) | 2.57× (2.50–2.65) | 2.42× (2.39–2.45) | 3.19× (3.10–3.28) | 2.91× (2.85–2.96) | 5.19× (5.08–5.31) | 5.43× (5.29–5.56) | 2.10× (2.07–2.13) | 2.12× (2.09–2.16) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 20.7 | 1.38× (1.35–1.42) | 2.41× (2.32–2.50) | 2.33× (2.25–2.41) | 2.93× (2.82–3.04) | 2.90× (2.81–3.01) | 6.30× (6.06–6.56) | 6.59× (6.36–6.82) | 2.55× (2.46–2.64) | 2.58× (2.49–2.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.156 s | 62 | 13.1 | 1.41× (1.38–1.44) | 3.11× (3.05–3.17) | 2.55× (2.53–2.58) | 2.70× (2.67–2.73) | 3.03× (2.98–3.08) | 3.80× (3.74–3.85) | 4.06× (4.01–4.11) | 1.47× (1.46–1.49) | 1.52× (1.50–1.54) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.096 s | 100 | 14.9 | 1.27× (1.24–1.31) | 2.83× (2.77–2.90) | 2.53× (2.47–2.58) | 3.54× (3.42–3.66) | 3.01× (2.94–3.09) | 6.31× (6.16–6.47) | 6.75× (6.56–6.95) | 2.45× (2.38–2.52) | 2.53× (2.46–2.61) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 19.9 | 1.46× (1.39–1.55) | 2.72× (2.58–2.87) | 2.50× (2.37–2.62) | 3.34× (3.24–3.44) | 2.97× (2.85–3.09) | 7.69× (7.42–7.97) | 8.23× (7.95–8.52) | 2.99× (2.88–3.10) | 3.09× (2.98–3.19) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 1.2 | 0.61× (0.59–0.62) | 3.05× (3.00–3.10) | 3.78× (3.71–3.84) | 3.17× (3.11–3.24) | 6.77× (6.40–7.17) | 2.32× (2.24–2.39) | 2.39× (2.32–2.46) | 1.03× (0.99–1.07) | 1.26× (1.00–1.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 727 | 1.2 | 0.53× (0.51–0.55) | 2.67× (2.40–2.97) | 5.10× (4.93–5.28) | 3.13× (3.06–3.20) | 6.69× (6.34–7.06) | 2.27× (2.20–2.33) | 2.34× (2.26–2.41) | 1.01× (0.96–1.06) | 1.23× (0.98–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 730 | 1.2 | 0.53× (0.52–0.55) | 2.84× (2.61–3.09) | 5.21× (5.13–5.29) | 3.12× (3.08–3.17) | 6.54× (6.36–6.71) | 2.27× (2.21–2.33) | 2.34× (2.27–2.41) | 1.01× (0.98–1.05) | 1.23× (0.98–1.55) | yes |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 1 | 0.073 s | 177 | 4.6 | 1.74× (1.65–1.83) | 3.91× (3.67–4.15) | 4.24× (4.01–4.49) | 4.41× (4.14–4.69) | 4.53× (4.26–4.82) | 4.80× (4.52–5.08) | 4.79× (4.54–5.06) | 1.44× (1.37–1.53) | 1.41× (1.33–1.49) | yes |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 2 | 0.047 s | 274 | 5.3 | 1.49× (1.37–1.61) | 3.35× (3.11–3.61) | 3.96× (3.63–4.33) | 4.48× (4.09–4.90) | 4.31× (3.98–4.67) | 7.25× (6.71–7.84) | 7.25× (6.73–7.81) | 2.18× (2.02–2.36) | 2.13× (1.96–2.32) | yes |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 4 | 0.042 s | 307 | 8.0 | 1.56× (1.48–1.64) | 2.73× (2.61–2.86) | 3.53× (3.40–3.66) | 3.79× (3.63–3.96) | 4.13× (3.92–4.35) | 8.47× (8.15–8.81) | 8.47× (8.11–8.85) | 2.55× (2.44–2.67) | 2.49× (2.39–2.59) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.48× (2.42–2.55) | 5.54× (5.40–5.69) | 5.90× (5.75–6.05) | 6.66× (6.49–6.84) | 5.95× (5.79–6.13) | 6.39× (6.21–6.58) | 6.16× (6.02–6.31) | 1.93× (1.88–1.98) | 1.78× (1.73–1.82) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.43× (2.41–2.46) | 5.48× (5.40–5.55) | 6.01× (5.90–6.12) | 6.78× (6.66–6.90) | 5.84× (5.71–5.96) | 6.06× (5.95–6.17) | 5.84× (5.72–5.96) | 1.83× (1.80–1.86) | 1.68× (1.65–1.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.54× (2.50–2.58) | 4.83× (4.74–4.92) | 4.99× (4.88–5.09) | 5.54× (5.42–5.65) | 4.89× (4.80–4.98) | 3.73× (3.67–3.79) | 3.60× (3.53–3.66) | 1.13× (1.11–1.15) | 1.04× (1.02–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.63) | 2.83× (2.77–2.90) | 3.96× (3.88–4.04) | 3.53× (3.45–3.62) | 5.83× (5.72–5.95) | 2.66× (2.62–2.71) | 2.68× (2.64–2.72) | 1.12× (1.10–1.15) | 0.89× (0.87–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.57–0.62) | 2.76× (2.67–2.85) | 5.05× (4.93–5.17) | 3.46× (3.39–3.53) | 5.63× (5.54–5.73) | 2.58× (2.53–2.64) | 2.60× (2.56–2.65) | 1.09× (1.06–1.12) | 0.87× (0.84–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.56–0.59) | 3.22× (3.15–3.29) | 5.12× (5.02–5.23) | 3.51× (3.45–3.57) | 5.78× (5.67–5.90) | 2.60× (2.55–2.65) | 2.62× (2.58–2.67) | 1.10× (1.07–1.13) | 0.87× (0.85–0.90) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.68–2.76) | 3.09× (3.01–3.17) | 3.63× (3.50–3.77) | 4.21× (4.11–4.31) | 4.61× (4.50–4.73) | 3.52× (3.41–3.63) | 2.83× (2.76–2.90) | 2.03× (1.99–2.07) | 1.94× (1.91–1.97) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.52× (2.45–2.59) | 2.88× (2.79–2.96) | 3.35× (3.24–3.46) | 3.77× (3.67–3.88) | 4.16× (3.93–4.41) | 3.02× (2.89–3.16) | 2.43× (2.35–2.51) | 1.74× (1.69–1.80) | 1.67× (1.63–1.71) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.22× (2.12–2.32) | 1.94× (1.89–2.00) | 2.39× (2.32–2.46) | 2.35× (2.28–2.43) | 2.46× (2.41–2.52) | 1.68× (1.63–1.72) | 1.35× (1.31–1.38) | 0.97× (0.94–0.99) | 0.92× (0.91–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.34–1.40) | 2.69× (2.66–2.71) | 2.42× (2.37–2.48) | 2.53× (2.49–2.58) | 2.78× (2.75–2.81) | 3.11× (3.05–3.18) | 3.25× (3.18–3.33) | 1.27× (1.25–1.28) | 1.25× (1.23–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.59× (2.53–2.64) | 2.24× (2.22–2.27) | 3.01× (2.94–3.08) | 2.59× (2.54–2.64) | 2.82× (2.78–2.87) | 2.95× (2.90–3.00) | 1.15× (1.14–1.16) | 1.13× (1.12–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.55–1.61) | 2.69× (2.64–2.75) | 2.25× (2.20–2.31) | 2.99× (2.91–3.06) | 2.61× (2.56–2.65) | 2.13× (2.09–2.17) | 2.23× (2.19–2.26) | 0.87× (0.85–0.88) | 0.85× (0.84–0.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.40× (1.37–1.44) | 3.06× (3.00–3.12) | 2.52× (2.49–2.54) | 2.68× (2.65–2.72) | 3.01× (2.96–3.06) | 3.77× (3.71–3.82) | 4.03× (3.98–4.07) | 1.47× (1.46–1.48) | 1.48× (1.46–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.26–1.33) | 2.89× (2.80–2.97) | 2.36× (2.31–2.42) | 3.28× (3.20–3.37) | 2.74× (2.68–2.81) | 3.35× (3.27–3.44) | 3.59× (3.48–3.69) | 1.31× (1.27–1.34) | 1.32× (1.28–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.56–1.67) | 3.04× (2.96–3.12) | 2.28× (2.20–2.37) | 3.18× (3.06–3.31) | 2.73× (2.64–2.81) | 2.48× (2.40–2.56) | 2.65× (2.57–2.73) | 0.97× (0.94–1.00) | 0.97× (0.95–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.65× (0.63–0.66) | 2.35× (2.30–2.41) | 3.37× (3.29–3.44) | 2.94× (2.89–2.99) | 6.16× (5.85–6.49) | 2.18× (2.12–2.24) | 2.24× (2.19–2.30) | 1.03× (0.99–1.07) | 0.81× (0.79–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.61) | 2.28× (2.23–2.34) | 4.20× (4.09–4.30) | 2.93× (2.87–3.00) | 6.03× (5.71–6.37) | 2.11× (2.05–2.17) | 2.17× (2.11–2.23) | 0.99× (0.95–1.04) | 0.79× (0.76–0.81) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.65× (2.58–2.72) | 4.26× (4.19–4.34) | 2.98× (2.92–3.03) | 5.98× (5.82–6.15) | 2.14× (2.08–2.21) | 2.21× (2.14–2.28) | 1.01× (0.98–1.05) | 0.80× (0.78–0.82) |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 1 | 1.72× (1.64–1.81) | 3.77× (3.57–3.98) | 4.12× (3.89–4.36) | 4.33× (4.07–4.61) | 4.45× (4.19–4.73) | 4.71× (4.45–4.99) | 4.71× (4.47–4.96) | 1.43× (1.36–1.51) | 1.36× (1.29–1.43) |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 2 | 1.60× (1.58–1.62) | 3.51× (3.39–3.63) | 3.92× (3.89–3.96) | 4.44× (4.32–4.56) | 4.03× (3.97–4.10) | 4.12× (4.07–4.17) | 4.12× (4.04–4.19) | 1.25× (1.24–1.27) | 1.19× (1.17–1.20) |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 4 | 1.83× (1.75–1.91) | 3.09× (2.99–3.20) | 3.35× (3.23–3.48) | 3.81× (3.67–3.97) | 3.49× (3.37–3.62) | 2.73× (2.64–2.82) | 2.73× (2.64–2.82) | 0.83× (0.80–0.86) | 0.79× (0.76–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.28× (0.28–0.28) | 1.05× (1.05–1.06) | 0.84× (0.84–0.84) | 11.92× (11.90–11.94) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 51 MB | 0.37× (0.37–0.38) | 0.51× (0.47–0.56) | 0.77× (0.75–0.78) | 0.65× (0.64–0.66) | 6.13× (6.05–6.21) | 0.14× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.55× (0.49–0.63) | 0.69× (0.68–0.71) | 0.61× (0.61–0.62) | 4.08× (4.02–4.14) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.40–0.44) | 1.70× (1.70–1.70) | 0.65× (0.65–0.66) | 8.01× (8.00–8.02) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.65× (0.62–0.69) | 1.63× (1.60–1.67) | 0.82× (0.81–0.84) | 5.95× (5.84–6.06) | 0.48× (0.47–0.49) | 0.51× (0.50–0.52) | 0.55× (0.54–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.67) | 0.72× (0.69–0.75) | 1.14× (1.10–1.18) | 0.71× (0.69–0.73) | 3.42× (3.32–3.52) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.17–1.18) | 0.73× (0.73–0.74) | 5.22× (5.20–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.47–0.51) | 0.39× (0.36–0.43) | 0.90× (0.86–0.94) | 0.70× (0.68–0.73) | 3.11× (2.97–3.26) | 0.19× (0.18–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.82× (0.81–0.84) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.22) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.38× (0.36–0.41) | 0.84× (0.82–0.87) | 0.70× (0.68–0.72) | 2.91× (2.83–2.99) | 0.18× (0.17–0.18) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.62× (0.61–0.64) | 0.49× (0.47–0.53) | 0.83× (0.82–0.84) | 0.84× (0.81–0.87) | 2.39× (2.36–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.77–0.78) | 5.32× (5.30–5.33) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.76–0.77) | 5.34× (5.32–5.35) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.66) | 6.40× (6.37–6.43) | 0.38× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 2 | 34 MB | 0.48× (0.47–0.49) | 0.45× (0.40–0.50) | 1.14× (1.12–1.17) | 0.72× (0.71–0.74) | 3.58× (3.50–3.66) | 0.22× (0.21–0.22) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.57 MB → 12.9 MB, 5.0×) | 4 | 55 MB | 0.51× (0.49–0.52) | 0.51× (0.46–0.56) | 0.93× (0.90–0.95) | 0.75× (0.72–0.78) | 2.36× (2.32–2.41) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 346 | 2.90 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 628 | 3.05 | 1.06× (1.02–1.09) | 91% | 1.04× (1.03–1.06) | 1.03× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 743 | 4.97 | 1.71× (1.65–1.77) | 54% | 1.49× (1.46–1.52) | 1.75× (1.73–1.77) |

**Best:** mibz2 at 4 threads (743 MB/s); lbzip2 at 4 threads (163 MB/s); pbz2 at 4 threads (284 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.73, mibz2 4t 1.14, lbzip2 1t 0.04, lbzip2 2t 0.22, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.36. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.82 | 0.03 | 0.09 | 0.282 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.28 | 11.4% |
| MTF + RLE2 | 0.42 | 16.8% |
| T-vector build | 0.39 | 15.4% |
| walk (the one each build uses) | 1.31 | 52.6% |
| of which: segment assembly | 0.07 | 2.7% |
| RLE1 | 0.07 | 2.6% |
| CRC | 0.03 | 1.3% |
| **total** | 2.50 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.468 | 1.231 | (in its total) | 2.63× (2.60–2.65) |  |
| walk | 1.558 | 2.997 | (in its total) | 1.93× (1.91–1.95) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.085 | 0.079 | (in its total) | 0.92× (0.91–0.93) |  |
| inverse BWT, total | 2.113 | 4.306 | 2.259 | 2.04× (2.03–2.06) | 1.08× (1.07–1.08) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h28m ± 10m42s |
| 4 threads | 40m41s ± 5m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m05s ± 4m18s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 30m05s ± 4m18s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 3.94 ns a byte of output at 4 threads (CPU, summed over them), 2.40 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,567,241 | 12,857,986 | \`31f79eb21ececf8e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
