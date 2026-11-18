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

The latest record: [`1edb29f`](<https://github.com/lmmx/mibz2/commit/1edb29fdc3d42dfddfc3d60e46456b611323760c>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44396806376>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.310 s | 677 | 3.9 | 2.89× (2.85–2.93) | 7.09× (6.96–7.23) | 7.39× (7.25–7.53) | 8.43× (8.30–8.57) | 7.52× (7.39–7.64) | 7.79× (7.69–7.89) | 7.30× (7.18–7.42) | 2.21× (2.19–2.22) | 2.53× (2.50–2.56) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.193 s | 1,087 | 4.3 | 2.80× (2.65–2.96) | 6.21× (5.90–6.55) | 7.34× (6.97–7.72) | 7.40× (7.02–7.81) | 6.60× (6.24–6.99) | 12.57× (11.89–13.29) | 11.79× (11.13–12.50) | 3.57× (3.37–3.77) | 4.09× (3.87–4.32) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.152 s | 1,380 | 6.5 | 2.74× (2.63–2.86) | 4.51× (4.26–4.77) | 5.86× (5.63–6.10) | 5.36× (5.18–5.55) | 5.37× (5.20–5.55) | 15.60× (15.00–16.21) | 14.63× (14.04–15.24) | 4.42× (4.26–4.59) | 5.07× (4.88–5.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,404 | 2.1 | 0.58× (0.56–0.59) | 3.55× (3.45–3.64) | 4.68× (4.25–5.14) | 3.76× (3.65–3.88) | 6.55× (6.37–6.73) | 2.82× (2.73–2.92) | 2.88× (2.71–3.05) | 0.99× (0.97–1.02) | 2.19× (2.04–2.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,338 | 2.2 | 0.52× (0.51–0.54) | 2.96× (2.75–3.19) | 5.94× (5.43–6.50) | 3.68× (3.58–3.78) | 6.51× (6.06–7.00) | 2.75× (2.68–2.81) | 2.80× (2.62–3.00) | 0.97× (0.94–0.99) | 2.13× (1.99–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,307 | 2.2 | 0.50× (0.48–0.53) | 3.04× (2.81–3.30) | 5.53× (5.25–5.82) | 3.55× (3.40–3.71) | 6.16× (5.95–6.37) | 2.63× (2.53–2.74) | 2.68× (2.52–2.86) | 0.93× (0.89–0.96) | 2.04× (1.90–2.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,054 | 2.7 | 2.99× (2.95–3.03) | 3.75× (3.63–3.87) | 4.43× (4.38–4.49) | 4.69× (4.59–4.78) | 5.41× (5.32–5.50) | 3.05× (2.99–3.11) | 2.62× (2.60–2.65) | 2.19× (2.13–2.24) | 3.26× (3.22–3.30) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.014 s | 1,420 | 3.6 | 2.10× (1.91–2.31) | 2.82× (2.35–3.38) | 3.57× (3.00–4.25) | 3.42× (2.93–3.99) | 5.14× (4.38–6.02) | 3.74× (3.19–4.39) | 3.22× (2.73–3.78) | 2.68× (2.31–3.11) | 4.00× (3.41–4.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,167 | 7.3 | 1.65× (1.61–1.70) | 1.89× (1.84–1.93) | 2.71× (2.57–2.86) | 2.03× (1.94–2.13) | 4.17× (4.09–4.27) | 3.38× (3.27–3.49) | 2.90× (2.82–2.99) | 2.42× (2.34–2.50) | 3.61× (3.50–3.71) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.084 s | 95 | 27.8 | 1.35× (1.33–1.38) | 3.02× (2.95–3.08) | 2.77× (2.70–2.85) | 2.68× (2.64–2.72) | 3.03× (2.97–3.08) | 3.42× (3.36–3.48) | 3.41× (3.35–3.47) | 1.28× (1.26–1.30) | 1.48× (1.45–1.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.054 s | 147 | 32.3 | 1.23× (1.19–1.28) | 2.74× (2.59–2.90) | 2.58× (2.40–2.78) | 3.16× (3.03–3.30) | 3.10× (2.93–3.27) | 5.29× (5.09–5.50) | 5.27× (5.06–5.49) | 1.98× (1.91–2.05) | 2.29× (2.21–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.044 s | 181 | 43.2 | 1.25× (1.23–1.28) | 2.38× (2.33–2.43) | 2.47× (2.34–2.60) | 2.80× (2.71–2.89) | 3.10× (2.95–3.25) | 6.51× (6.30–6.72) | 6.48× (6.26–6.72) | 2.44× (2.36–2.52) | 2.82× (2.71–2.95) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 24.0 | 1.49× (1.47–1.52) | 3.69× (3.62–3.75) | 3.07× (2.98–3.16) | 3.06× (3.03–3.09) | 3.49× (3.45–3.53) | 4.39× (4.36–4.42) | 4.69× (4.61–4.77) | 1.59× (1.57–1.60) | 1.87× (1.81–1.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.056 s | 172 | 28.2 | 1.29× (1.22–1.37) | 3.16× (3.04–3.29) | 2.71× (2.58–2.85) | 3.74× (3.54–3.96) | 3.29× (3.16–3.42) | 6.78× (6.42–7.15) | 7.24× (6.86–7.64) | 2.45× (2.32–2.58) | 2.89× (2.73–3.06) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 204 | 39.0 | 1.35× (1.29–1.41) | 2.63× (2.56–2.69) | 2.49× (2.39–2.59) | 3.39× (3.26–3.52) | 3.10× (3.01–3.20) | 8.08× (7.86–8.31) | 8.63× (8.36–8.92) | 2.92× (2.82–3.02) | 3.44× (3.33–3.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,064 | 2.8 | 0.64× (0.62–0.66) | 3.26× (3.19–3.33) | 4.52× (4.42–4.63) | 3.27× (3.15–3.39) | 7.25× (7.10–7.41) | 2.41× (2.36–2.45) | 2.43× (2.38–2.49) | 0.99× (0.97–1.02) | 2.18× (2.13–2.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 990 | 3.1 | 0.52× (0.51–0.53) | 3.05× (3.00–3.11) | 5.25× (5.16–5.35) | 3.06× (3.02–3.10) | 6.74× (6.62–6.86) | 2.25× (2.22–2.28) | 2.27× (2.24–2.30) | 0.93× (0.92–0.94) | 2.04× (2.00–2.07) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 971 | 3.0 | 0.51× (0.51–0.52) | 3.02× (2.96–3.07) | 5.12× (4.99–5.25) | 3.10× (3.00–3.21) | 7.01× (6.30–7.79) | 2.21× (2.17–2.26) | 2.23× (2.19–2.28) | 0.91× (0.89–0.94) | 2.00× (1.97–2.04) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.044 s | 296 | 9.1 | 1.77× (1.73–1.81) | 4.26× (4.15–4.39) | 4.70× (4.56–4.83) | 4.82× (4.63–5.03) | 5.02× (4.89–5.15) | 5.10× (4.97–5.23) | 4.86× (4.72–5.00) | 1.54× (1.48–1.60) | 1.82× (1.77–1.88) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.033 s | 403 | 11.8 | 1.42× (1.35–1.49) | 3.37× (3.14–3.61) | 4.03× (3.82–4.25) | 4.49× (4.23–4.78) | 4.42× (4.26–4.58) | 7.01× (6.70–7.33) | 6.68× (6.37–7.01) | 2.12× (2.02–2.21) | 2.51× (2.39–2.62) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.031 s | 420 | 18.4 | 1.28× (1.16–1.42) | 2.30× (2.16–2.44) | 3.09× (2.94–3.25) | 3.11× (2.97–3.27) | 3.84× (3.52–4.19) | 7.15× (6.78–7.54) | 6.81× (6.47–7.18) | 2.16× (2.07–2.25) | 2.56× (2.41–2.71) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.84–2.92) | 7.04× (6.91–7.18) | 7.34× (7.21–7.48) | 8.39× (8.26–8.52) | 7.48× (7.35–7.60) | 7.75× (7.65–7.85) | 7.27× (7.15–7.39) | 2.20× (2.18–2.22) | 2.51× (2.48–2.54) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.63× (2.53–2.75) | 6.45× (6.20–6.72) | 6.82× (6.57–7.08) | 7.77× (7.47–8.08) | 6.67× (6.39–6.96) | 6.78× (6.50–7.06) | 6.36× (6.08–6.64) | 1.93× (1.85–2.01) | 2.20× (2.11–2.28) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.69× (2.61–2.78) | 5.11× (4.84–5.39) | 5.62× (5.44–5.80) | 6.08× (5.90–6.26) | 5.31× (5.17–5.45) | 4.54× (4.40–4.68) | 4.26× (4.12–4.41) | 1.29× (1.25–1.33) | 1.47× (1.43–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.11× (3.04–3.18) | 4.09× (3.80–4.41) | 3.49× (3.38–3.60) | 6.00× (5.82–6.18) | 2.65× (2.56–2.73) | 2.70× (2.54–2.87) | 1.00× (0.97–1.02) | 1.87× (1.80–1.94) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.82× (2.75–2.90) | 5.15× (4.77–5.55) | 3.38× (3.31–3.46) | 5.87× (5.47–6.30) | 2.53× (2.47–2.58) | 2.57× (2.42–2.74) | 0.95× (0.92–0.98) | 1.79× (1.71–1.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.57) | 2.98× (2.89–3.08) | 4.88× (4.68–5.09) | 3.35× (3.22–3.48) | 5.68× (5.52–5.85) | 2.49× (2.41–2.58) | 2.54× (2.40–2.69) | 0.94× (0.90–0.98) | 1.76× (1.68–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.83× (2.79–2.88) | 3.24× (3.15–3.33) | 3.83× (3.76–3.89) | 4.40× (4.30–4.50) | 5.06× (4.97–5.15) | 2.89× (2.83–2.95) | 2.50× (2.47–2.53) | 2.09× (2.05–2.14) | 2.80× (2.77–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.30× (2.18–2.43) | 2.66× (2.41–2.94) | 3.00× (2.73–3.30) | 3.36× (3.06–3.69) | 3.81× (3.49–4.16) | 2.10× (1.92–2.29) | 1.81× (1.65–1.99) | 1.52× (1.41–1.65) | 2.03× (1.85–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.80–1.92) | 1.65× (1.58–1.71) | 1.98× (1.89–2.07) | 1.97× (1.88–2.06) | 2.18× (2.09–2.27) | 1.05× (1.00–1.11) | 0.91× (0.87–0.95) | 0.76× (0.73–0.80) | 1.02× (0.98–1.06) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.32–1.37) | 2.93× (2.88–2.99) | 2.67× (2.61–2.73) | 2.65× (2.60–2.69) | 2.99× (2.92–3.05) | 3.37× (3.31–3.44) | 3.36× (3.29–3.43) | 1.27× (1.25–1.30) | 1.42× (1.39–1.44) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.23× (1.22–1.25) | 2.72× (2.63–2.82) | 2.37× (2.27–2.47) | 3.04× (3.01–3.06) | 2.74× (2.59–2.89) | 2.92× (2.88–2.97) | 2.91× (2.87–2.96) | 1.11× (1.09–1.12) | 1.23× (1.21–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.34× (1.29–1.39) | 2.51× (2.46–2.55) | 2.13× (1.98–2.29) | 2.71× (2.62–2.80) | 2.58× (2.52–2.64) | 2.17× (2.10–2.24) | 2.16× (2.09–2.24) | 0.82× (0.80–0.85) | 0.91× (0.88–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.46–1.51) | 3.60× (3.54–3.65) | 2.98× (2.90–3.06) | 3.02× (2.99–3.06) | 3.45× (3.41–3.49) | 4.33× (4.29–4.36) | 4.62× (4.54–4.71) | 1.57× (1.56–1.59) | 1.79× (1.72–1.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.29–1.35) | 3.21× (3.13–3.29) | 2.56× (2.49–2.63) | 3.61× (3.49–3.73) | 3.00× (2.94–3.06) | 3.69× (3.60–3.79) | 3.95× (3.84–4.06) | 1.34× (1.31–1.38) | 1.53× (1.48–1.58) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.51× (1.46–1.57) | 2.90× (2.83–2.97) | 2.17× (2.12–2.22) | 3.11× (3.06–3.15) | 2.79× (2.75–2.84) | 2.66× (2.61–2.71) | 2.84× (2.77–2.91) | 0.97× (0.94–0.99) | 1.10× (1.06–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.68–0.71) | 2.62× (2.57–2.67) | 3.44× (3.39–3.49) | 2.96× (2.87–3.04) | 6.41× (6.28–6.54) | 2.23× (2.18–2.27) | 2.23× (2.19–2.28) | 1.00× (0.97–1.02) | 1.53× (1.50–1.57) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.58–0.61) | 2.36× (2.33–2.39) | 3.99× (3.89–4.10) | 2.78× (2.74–2.83) | 5.83× (5.70–5.98) | 2.03× (1.99–2.06) | 2.04× (1.99–2.08) | 0.91× (0.89–0.93) | 1.40× (1.37–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.60× (2.45–2.76) | 4.04× (3.97–4.11) | 2.92× (2.82–3.02) | 6.28× (5.66–6.98) | 2.07× (2.03–2.11) | 2.08× (2.03–2.12) | 0.93× (0.90–0.96) | 1.42× (1.40–1.45) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.74× (1.70–1.78) | 4.06× (3.95–4.17) | 4.44× (4.33–4.56) | 4.69× (4.50–4.89) | 4.88× (4.76–5.01) | 4.95× (4.83–5.08) | 4.72× (4.59–4.87) | 1.52× (1.46–1.58) | 1.64× (1.60–1.69) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.47× (1.42–1.51) | 3.40× (3.26–3.54) | 3.73× (3.60–3.87) | 4.19× (4.08–4.30) | 3.89× (3.82–3.98) | 3.89× (3.77–4.01) | 3.71× (3.59–3.82) | 1.19× (1.16–1.22) | 1.29× (1.26–1.32) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.47× (1.39–1.56) | 2.62× (2.49–2.75) | 2.87× (2.74–2.99) | 3.28× (3.13–3.43) | 3.24× (3.00–3.49) | 2.51× (2.39–2.63) | 2.39× (2.28–2.50) | 0.77× (0.73–0.81) | 0.83× (0.80–0.86) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.77× (0.76–0.79) | 10.99× (10.98–11.00) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.42) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.32× (0.30–0.34) | 0.50× (0.42–0.58) | 0.65× (0.62–0.68) | 0.55× (0.53–0.58) | 5.24× (5.00–5.50) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 90 MB | 0.34× (0.33–0.36) | 0.52× (0.42–0.64) | 0.61× (0.58–0.63) | 0.53× (0.51–0.55) | 3.53× (3.40–3.67) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.02–8.06) | 0.65× (0.65–0.66) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.68× (0.65–0.71) | 1.60× (1.55–1.65) | 0.81× (0.79–0.84) | 5.81× (5.65–5.99) | 0.47× (0.45–0.49) | 0.49× (0.48–0.50) | 0.53× (0.51–0.55) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.62–0.65) | 0.70× (0.65–0.75) | 1.15× (1.12–1.18) | 0.71× (0.69–0.73) | 3.36× (3.29–3.44) | 0.26× (0.26–0.27) | 0.27× (0.27–0.28) | 0.30× (0.29–0.30) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.43) | 0.89× (0.87–0.92) | 0.69× (0.65–0.74) | 3.10× (3.02–3.19) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.46) | 0.86× (0.83–0.89) | 0.82× (0.80–0.85) | 2.52× (2.46–2.59) | 0.15× (0.15–0.16) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.51) | 0.39× (0.35–0.42) | 0.85× (0.81–0.90) | 0.73× (0.68–0.77) | 2.94× (2.80–3.09) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.46× (0.43–0.50) | 0.79× (0.78–0.80) | 0.81× (0.78–0.84) | 2.28× (2.24–2.32) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.20–0.23) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.30× (5.28–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.32× (5.30–5.33) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.45) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.65–0.66) | 6.36× (6.35–6.38) | 0.39× (0.39–0.40) | 0.42× (0.41–0.42) | 0.62× (0.62–0.62) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.40× (0.36–0.45) | 1.14× (1.12–1.17) | 0.74× (0.72–0.76) | 3.63× (3.58–3.67) | 0.22× (0.22–0.23) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.18–0.18) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 55 MB | 0.52× (0.49–0.55) | 0.48× (0.44–0.53) | 0.95× (0.90–0.99) | 0.78× (0.75–0.81) | 2.39× (2.30–2.49) | 0.14× (0.14–0.15) | 0.15× (0.14–0.16) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 677 | 1.49 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,087 | 1.66 | 1.14× (1.10–1.19) | 81% | 1.05× (1.03–1.07) | 1.05× (1.02–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,380 | 2.50 | 1.71× (1.66–1.76) | 50% | 1.24× (1.17–1.31) | 1.60× (1.53–1.66) |

**Best:** mibz2 at 4 threads (1,380 MB/s); lbzip2 at 4 threads (308 MB/s); pbz2 at 4 threads (494 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.87, mibz2 4t 1.25, lbzip2 1t 0.07, lbzip2 2t 0.33, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.57. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.39 | 0.01 | 0.07 | 0.152 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.61 | 14.5% |
| MTF + RLE2 | 0.89 | 21.3% |
| T-vector build | 0.79 | 18.7% |
| walk (the one each build uses) | 1.65 | 39.3% |
| of which: segment assembly | 0.07 | 1.6% |
| RLE1 | 0.09 | 2.2% |
| CRC | 0.15 | 3.6% |
| **total** | 4.19 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.298 | 0.715 | (in its total) | 2.42× (2.38–2.45) |  |
| walk | 0.616 | 1.588 | (in its total) | 2.61× (2.56–2.65) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.030 | (in its total) | 1.17× (1.14–1.21) |  |
| inverse BWT, total | 0.940 | 2.331 | 1.324 | 2.51× (2.46–2.55) | 1.43× (1.41–1.45) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m56s ± 5m08s |
| 4 threads | 22m27s ± 3m14s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m09s ± 2m02s |
| the scanner alone (its serial part) | 8m35s ± 19.7s |
| the workers alone, over 4 | 14m09s ± 2m02s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 1.85 ns a byte of output at 4 threads (CPU, summed over them), 1.19 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,747 | 13,092,730 | \`9b1907f0616f895e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
