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

The latest record: [`478b92e`](<https://github.com/lmmx/mibz2/commit/478b92ea82af3aa1f42e5f80bff959c98b4f7bb1>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46110221441>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.381 s | 550 | 6.8 | 3.15× (3.13–3.17) | 7.25× (7.20–7.29) | 7.47× (7.42–7.52) | 8.53× (8.48–8.59) | 7.49× (7.44–7.53) | 7.85× (7.80–7.89) | 7.56× (7.52–7.61) | 2.34× (2.33–2.36) | 2.63× (2.61–2.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.214 s | 980 | 7.2 | 3.22× (3.09–3.36) | 6.64× (6.26–7.05) | 7.62× (7.25–8.01) | 8.04× (7.60–8.50) | 7.00× (6.66–7.37) | 13.74× (13.06–14.46) | 13.25× (12.59–13.94) | 4.11× (3.90–4.32) | 4.61× (4.38–4.86) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.176 s | 1,194 | 11.4 | 3.21× (3.14–3.27) | 4.81× (4.72–4.91) | 5.90× (5.79–6.01) | 5.86× (5.79–5.93) | 5.72× (5.66–5.78) | 17.06× (16.86–17.26) | 16.44× (16.24–16.65) | 5.10× (5.04–5.15) | 5.73× (5.66–5.80) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,124 | 3.6 | 0.57× (0.56–0.59) | 3.42× (3.37–3.48) | 4.58× (4.50–4.65) | 3.57× (3.51–3.62) | 6.15× (6.06–6.24) | 2.66× (2.62–2.71) | 2.66× (2.62–2.70) | 0.96× (0.95–0.98) | 1.77× (1.65–1.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,084 | 3.9 | 0.52× (0.51–0.53) | 2.76× (2.70–2.82) | 5.53× (5.42–5.64) | 3.45× (3.38–3.51) | 5.79× (5.69–5.90) | 2.56× (2.51–2.62) | 2.56× (2.51–2.62) | 0.93× (0.91–0.95) | 1.70× (1.61–1.80) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,058 | 3.9 | 0.51× (0.50–0.52) | 2.71× (2.66–2.75) | 5.40× (5.30–5.50) | 3.37× (3.30–3.44) | 5.74× (5.62–5.85) | 2.50× (2.46–2.55) | 2.50× (2.46–2.55) | 0.91× (0.89–0.92) | 1.66× (1.58–1.75) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 767 | 5.1 | 2.80× (2.78–2.82) | 3.20× (3.18–3.21) | 3.96× (3.86–4.05) | 4.26× (4.24–4.28) | 4.69× (4.65–4.73) | 2.89× (2.86–2.92) | 2.36× (2.35–2.37) | 2.16× (2.15–2.17) | 3.22× (3.13–3.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,206 | 6.0 | 2.41× (2.27–2.56) | 3.17× (2.96–3.39) | 4.70× (4.33–5.11) | 3.87× (3.69–4.06) | 5.56× (5.29–5.85) | 4.47× (4.23–4.73) | 3.66× (3.47–3.86) | 3.34× (3.17–3.53) | 4.99× (4.72–5.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,083 | 11.9 | 2.07× (1.99–2.14) | 2.07× (1.89–2.28) | 3.31× (3.14–3.49) | 2.50× (2.33–2.68) | 4.56× (4.38–4.75) | 4.08× (3.92–4.23) | 3.33× (3.21–3.46) | 3.05× (2.93–3.16) | 4.55× (4.34–4.76) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.100 s | 80 | 46.9 | 1.51× (1.50–1.52) | 2.87× (2.82–2.91) | 2.65× (2.60–2.69) | 2.76× (2.74–2.79) | 3.06× (3.04–3.09) | 3.28× (3.26–3.30) | 3.43× (3.40–3.46) | 1.37× (1.36–1.38) | 1.52× (1.49–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 130 | 52.4 | 1.40× (1.32–1.49) | 2.70× (2.53–2.88) | 2.59× (2.44–2.75) | 3.38× (3.22–3.55) | 3.14× (2.99–3.30) | 5.30× (5.04–5.58) | 5.54× (5.26–5.84) | 2.21× (2.10–2.33) | 2.46× (2.37–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.050 s | 161 | 67.6 | 1.48× (1.44–1.52) | 2.43× (2.32–2.54) | 2.38× (2.27–2.50) | 3.00× (2.83–3.17) | 3.22× (3.13–3.32) | 6.62× (6.43–6.83) | 6.92× (6.72–7.13) | 2.76× (2.68–2.85) | 3.08× (3.02–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.110 s | 88 | 42.8 | 1.64× (1.62–1.65) | 3.28× (3.23–3.34) | 2.88× (2.84–2.92) | 3.02× (2.99–3.05) | 3.34× (3.31–3.38) | 3.99× (3.95–4.03) | 4.38× (4.34–4.43) | 1.61× (1.60–1.63) | 1.93× (1.92–1.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.065 s | 147 | 48.4 | 1.52× (1.49–1.54) | 3.15× (2.93–3.39) | 2.83× (2.77–2.89) | 3.98× (3.88–4.09) | 3.42× (3.36–3.47) | 6.70× (6.59–6.80) | 7.36× (7.23–7.48) | 2.71× (2.67–2.75) | 3.25× (3.20–3.30) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 172 | 66.5 | 1.60× (1.55–1.64) | 2.50× (2.44–2.56) | 2.53× (2.46–2.60) | 3.44× (3.35–3.54) | 3.21× (3.12–3.29) | 7.91× (7.71–8.12) | 8.69× (8.47–8.92) | 3.20× (3.12–3.28) | 3.84× (3.75–3.93) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 5.0 | 0.63× (0.62–0.63) | 3.49× (3.46–3.52) | 4.39× (4.35–4.43) | 3.11× (3.08–3.14) | 6.78× (6.73–6.83) | 2.30× (2.28–2.32) | 2.35× (2.34–2.37) | 0.95× (0.94–0.96) | 1.75× (1.74–1.77) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 787 | 5.6 | 0.51× (0.50–0.51) | 2.43× (2.40–2.46) | 4.88× (4.81–4.94) | 2.89× (2.85–2.93) | 6.26× (6.13–6.39) | 2.12× (2.10–2.15) | 2.17× (2.14–2.20) | 0.88× (0.87–0.89) | 1.62× (1.60–1.64) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 5.5 | 0.51× (0.50–0.52) | 2.40× (2.36–2.44) | 4.81× (4.73–4.89) | 2.88× (2.83–2.92) | 6.15× (6.04–6.26) | 2.09× (2.05–2.13) | 2.14× (2.10–2.18) | 0.86× (0.85–0.88) | 1.59× (1.57–1.62) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 0.051 s | 292 | 13.1 | 2.05× (2.04–2.07) | 4.15× (4.13–4.16) | 4.67× (4.60–4.74) | 4.94× (4.91–4.96) | 5.08× (5.05–5.11) | 4.84× (4.82–4.86) | 5.01× (4.93–5.09) | 1.68× (1.67–1.69) | 1.97× (1.96–1.98) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 0.035 s | 428 | 16.6 | 1.60× (1.53–1.67) | 3.38× (3.23–3.55) | 4.20× (3.97–4.45) | 4.74× (4.52–4.98) | 4.64× (4.38–4.92) | 7.00× (6.73–7.28) | 7.25× (6.93–7.58) | 2.43× (2.33–2.53) | 2.85× (2.74–2.97) | yes |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 0.031 s | 487 | 25.3 | 1.71× (1.63–1.78) | 2.69× (2.58–2.81) | 3.64× (3.54–3.74) | 3.90× (3.80–4.01) | 4.21× (4.10–4.31) | 8.01× (7.82–8.20) | 8.29× (8.06–8.52) | 2.78× (2.71–2.84) | 3.26× (3.19–3.33) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.14× (3.12–3.16) | 7.21× (7.16–7.25) | 7.42× (7.37–7.47) | 8.50× (8.44–8.55) | 7.45× (7.41–7.50) | 7.81× (7.77–7.86) | 7.53× (7.49–7.58) | 2.34× (2.32–2.35) | 2.62× (2.60–2.63) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.03× (3.02–3.04) | 6.87× (6.82–6.92) | 7.15× (7.12–7.18) | 8.39× (8.34–8.44) | 7.02× (6.99–7.05) | 7.32× (7.28–7.35) | 7.05× (7.02–7.09) | 2.19× (2.18–2.20) | 2.45× (2.43–2.47) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.02× (2.97–3.08) | 5.10× (5.02–5.17) | 5.45× (5.37–5.52) | 6.22× (6.14–6.29) | 5.33× (5.26–5.40) | 4.65× (4.60–4.71) | 4.49× (4.43–4.54) | 1.39× (1.37–1.41) | 1.56× (1.53–1.58) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.91× (2.87–2.96) | 3.91× (3.82–4.00) | 3.34× (3.29–3.39) | 5.69× (5.62–5.77) | 2.51× (2.48–2.55) | 2.51× (2.48–2.55) | 0.97× (0.95–0.98) | 1.63× (1.61–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.61× (2.56–2.66) | 4.67× (4.59–4.74) | 3.14× (3.09–3.20) | 5.20× (5.11–5.30) | 2.34× (2.30–2.38) | 2.34× (2.30–2.39) | 0.90× (0.88–0.92) | 1.52× (1.49–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.55) | 2.85× (2.78–2.93) | 4.70× (4.61–4.78) | 3.13× (3.06–3.20) | 5.23× (5.12–5.33) | 2.33× (2.29–2.37) | 2.33× (2.29–2.37) | 0.90× (0.88–0.91) | 1.51× (1.49–1.54) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.68× (2.66–2.70) | 2.94× (2.92–2.95) | 3.46× (3.45–3.48) | 4.05× (4.04–4.07) | 4.45× (4.42–4.49) | 2.76× (2.75–2.77) | 2.27× (2.26–2.28) | 2.09× (2.08–2.09) | 2.76× (2.68–2.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.44× (2.39–2.50) | 2.71× (2.66–2.76) | 3.62× (3.53–3.70) | 3.62× (3.54–3.69) | 3.94× (3.86–4.02) | 2.34× (2.29–2.39) | 1.93× (1.89–1.97) | 1.77× (1.73–1.81) | 2.34× (2.29–2.39) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.08× (1.90–2.27) | 1.91× (1.85–1.99) | 2.42× (2.34–2.50) | 2.24× (2.16–2.33) | 2.33× (2.26–2.39) | 1.20× (1.16–1.23) | 0.99× (0.96–1.01) | 0.91× (0.88–0.93) | 1.20× (1.15–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.50× (1.49–1.51) | 2.81× (2.78–2.83) | 2.58× (2.52–2.63) | 2.73× (2.71–2.76) | 3.03× (3.00–3.05) | 3.24× (3.22–3.26) | 3.38× (3.35–3.42) | 1.36× (1.35–1.37) | 1.49× (1.48–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.40× (1.38–1.43) | 2.65× (2.60–2.71) | 2.36× (2.31–2.41) | 3.22× (3.16–3.29) | 2.77× (2.71–2.82) | 2.90× (2.85–2.95) | 3.03× (2.97–3.10) | 1.22× (1.19–1.24) | 1.34× (1.31–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.67× (1.65–1.69) | 2.58× (2.54–2.62) | 2.18× (2.14–2.22) | 2.98× (2.91–3.05) | 2.78× (2.74–2.82) | 2.26× (2.22–2.30) | 2.36× (2.32–2.40) | 0.95× (0.93–0.96) | 1.04× (1.02–1.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.63× (1.61–1.64) | 3.21× (3.18–3.25) | 2.80× (2.75–2.84) | 2.99× (2.96–3.02) | 3.31× (3.27–3.34) | 3.94× (3.90–3.98) | 4.33× (4.28–4.37) | 1.60× (1.59–1.62) | 1.86× (1.84–1.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.51× (1.50–1.52) | 3.00× (2.97–3.02) | 2.57× (2.54–2.61) | 3.62× (3.59–3.65) | 3.01× (2.99–3.02) | 3.51× (3.49–3.53) | 3.85× (3.83–3.88) | 1.43× (1.42–1.43) | 1.66× (1.64–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.74× (1.71–1.77) | 2.73× (2.69–2.78) | 2.16× (2.13–2.19) | 3.13× (3.07–3.20) | 2.83× (2.79–2.88) | 2.52× (2.48–2.56) | 2.77× (2.72–2.82) | 1.03× (1.01–1.04) | 1.19× (1.17–1.21) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.68) | 2.47× (2.46–2.49) | 3.41× (3.39–3.43) | 2.85× (2.84–2.86) | 6.07× (6.02–6.12) | 2.14× (2.13–2.15) | 2.19× (2.18–2.19) | 0.96× (0.96–0.97) | 1.38× (1.37–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.55–0.56) | 2.14× (2.12–2.15) | 3.76× (3.73–3.80) | 2.59× (2.56–2.62) | 5.37× (5.27–5.46) | 1.89× (1.87–1.91) | 1.93× (1.90–1.95) | 0.85× (0.84–0.85) | 1.22× (1.20–1.23) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.55) | 2.36× (2.32–2.39) | 3.77× (3.71–3.83) | 2.66× (2.62–2.70) | 5.45× (5.36–5.54) | 1.92× (1.89–1.95) | 1.96× (1.93–2.00) | 0.86× (0.85–0.88) | 1.24× (1.22–1.26) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 2.02× (2.01–2.03) | 3.99× (3.97–4.01) | 4.43× (4.41–4.46) | 4.81× (4.79–4.83) | 4.95× (4.92–4.98) | 4.71× (4.70–4.73) | 4.88× (4.80–4.96) | 1.66× (1.65–1.66) | 1.85× (1.84–1.86) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 1.68× (1.67–1.68) | 3.28× (3.27–3.30) | 3.67× (3.64–3.70) | 4.34× (4.31–4.37) | 4.05× (3.93–4.18) | 3.73× (3.70–3.75) | 3.86× (3.80–3.92) | 1.31× (1.30–1.32) | 1.46× (1.45–1.47) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 1.77× (1.70–1.84) | 2.64× (2.57–2.70) | 2.95× (2.89–3.02) | 3.38× (3.32–3.45) | 3.17× (3.12–3.22) | 2.45× (2.41–2.49) | 2.53× (2.49–2.58) | 0.86× (0.85–0.87) | 0.96× (0.94–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.78–0.79) | 11.00× (10.98–11.01) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.42× (0.36–0.50) | 0.72× (0.70–0.74) | 0.61× (0.59–0.63) | 5.73× (5.56–5.92) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.38) | 0.51× (0.42–0.63) | 0.63× (0.60–0.66) | 0.57× (0.54–0.59) | 3.71× (3.54–3.88) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.10–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.17× (0.17–0.18) | 1.95× (1.95–1.95) | 0.92× (0.92–0.92) | 4.83× (4.82–4.84) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.94–1.95) | 0.92× (0.92–0.92) | 4.83× (4.81–4.84) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.68× (1.68–1.69) | 0.66× (0.66–0.66) | 7.94× (7.92–7.96) | 0.65× (0.64–0.65) | 0.68× (0.67–0.68) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.67–0.71) | 0.68× (0.62–0.74) | 1.65× (1.61–1.69) | 0.82× (0.80–0.85) | 5.98× (5.82–6.14) | 0.48× (0.47–0.50) | 0.51× (0.49–0.52) | 0.55× (0.54–0.57) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.68× (0.64–0.72) | 0.78× (0.72–0.84) | 1.21× (1.15–1.26) | 0.75× (0.71–0.79) | 3.58× (3.40–3.77) | 0.28× (0.27–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.13–5.17) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.43) | 0.89× (0.86–0.92) | 0.72× (0.70–0.74) | 3.10× (2.99–3.20) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.86× (0.85–0.88) | 0.81× (0.80–0.83) | 2.52× (2.47–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.17–1.18) | 0.83× (0.82–0.84) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.46–0.49) | 0.36× (0.33–0.39) | 0.83× (0.81–0.85) | 0.70× (0.67–0.73) | 2.85× (2.77–2.92) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.59–0.61) | 0.46× (0.44–0.48) | 0.79× (0.78–0.80) | 0.79× (0.76–0.83) | 2.29× (2.27–2.31) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.06× (1.06–1.06) | 0.75× (0.75–0.76) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.29) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.20× (6.18–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.23) | 1.85× (1.85–1.86) | 0.89× (0.89–0.89) | 6.15× (6.14–6.16) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 1.05× (1.05–1.05) | 0.15× (0.15–0.15) |
| `text` (2.26 MB → 15 MB, 6.6×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.40× (1.39–1.41) | 0.66× (0.66–0.67) | 6.44× (6.40–6.48) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.26 MB → 15 MB, 6.6×) | 2 | 36 MB | 0.47× (0.46–0.48) | 0.39× (0.36–0.42) | 1.08× (1.05–1.11) | 0.71× (0.70–0.73) | 3.51× (3.44–3.59) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.35) | 0.17× (0.17–0.17) |
| `text` (2.26 MB → 15 MB, 6.6×) | 4 | 55 MB | 0.51× (0.49–0.54) | 0.46× (0.42–0.51) | 0.93× (0.91–0.95) | 0.76× (0.73–0.79) | 2.40× (2.36–2.44) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 550 | 1.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 980 | 1.96 | 1.07× (1.06–1.07) | 88% | 1.02× (1.01–1.02) | 1.03× (1.03–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,194 | 3.09 | 1.68× (1.65–1.71) | 54% | 1.19× (1.18–1.20) | 1.62× (1.60–1.63) |

**Best:** mibz2 at 4 threads (1,194 MB/s); lbzip2 at 4 threads (249 MB/s); pbz2 at 4 threads (373 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.06, lbzip2 1t 0.06, lbzip2 2t 0.26, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.51 | 0.01 | 0.07 | 0.176 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3693 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.80 | 9.7% |
| MTF + RLE2 | 1.49 | 18.0% |
| T-vector build | 1.48 | 18.0% |
| walk (the one each build uses) | 4.03 | 48.8% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.1% |
| **total** | 8.26 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.060 | (in its total) | 2.71× (2.70–2.71) |  |
| walk | 1.050 | 2.057 | (in its total) | 1.96× (1.95–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.040 | 0.040 | (in its total) | 1.02× (1.01–1.04) |  |
| inverse BWT, total | 1.481 | 3.156 | 1.904 | 2.13× (2.12–2.14) | 1.29× (1.28–1.30) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m37s ± 6m46s |
| 4 threads | 25m35s ± 3m36s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m20s ± 2m35s |
| the scanner alone (its serial part) | 7m25s ± 19.7s |
| the workers alone, over 4 | 18m20s ± 2m35s |

At 4 threads the workers bound it. Per byte: the scanner 4.3 ns a byte of bz2, the workers 2.40 ns a byte of output at 4 threads (CPU, summed over them), 1.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,260,864 | 15,030,132 | \`de55d3765b0096e1…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3693.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
