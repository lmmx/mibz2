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

The latest record: [`6fb7775`](<https://github.com/lmmx/mibz2/commit/6fb777541e3ce4613c34e14d8f5def7fbada8b4a>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44432693444>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.390 s | 537 | 7.3 | 3.41× (3.40–3.43) | 7.86× (7.81–7.91) | 8.37× (8.33–8.41) | 9.26× (9.19–9.33) | 8.42× (8.36–8.49) | 8.81× (8.77–8.86) | 8.52× (8.47–8.58) | 2.69× (2.68–2.70) | 2.39× (2.38–2.40) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.221 s | 949 | 7.9 | 3.29× (3.11–3.48) | 7.22× (6.88–7.57) | 8.61× (8.20–9.05) | 8.58× (8.16–9.02) | 7.75× (7.39–8.13) | 15.26× (14.56–16.00) | 14.76× (14.04–15.51) | 4.65× (4.43–4.88) | 4.14× (3.95–4.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.185 s | 1,133 | 12.3 | 3.36× (3.31–3.41) | 6.22× (6.13–6.32) | 7.29× (7.23–7.36) | 7.20× (7.11–7.30) | 7.05× (6.96–7.14) | 18.51× (18.26–18.76) | 17.90× (17.65–18.15) | 5.64× (5.57–5.71) | 5.02× (4.96–5.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,254 | 3.4 | 0.64× (0.61–0.68) | 3.25× (3.07–3.45) | 4.64× (4.37–4.93) | 3.86× (3.67–4.07) | 6.57× (6.24–6.92) | 2.92× (2.75–3.09) | 2.95× (2.79–3.12) | 1.14× (1.10–1.18) | 1.31× (1.24–1.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,238 | 3.5 | 0.57× (0.55–0.59) | 3.19× (3.06–3.32) | 6.46× (6.20–6.72) | 3.79× (3.67–3.91) | 6.44× (6.23–6.65) | 2.86× (2.75–2.97) | 2.89× (2.79–3.00) | 1.12× (1.09–1.15) | 1.29× (1.24–1.34) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,212 | 3.5 | 0.57× (0.54–0.61) | 3.16× (3.01–3.31) | 6.11× (5.82–6.41) | 3.76× (3.59–3.93) | 6.34× (6.09–6.61) | 2.82× (2.70–2.95) | 2.85× (2.73–2.98) | 1.10× (1.07–1.14) | 1.27× (1.21–1.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 636 | 6.4 | 2.54× (2.52–2.57) | 3.31× (3.29–3.33) | 3.98× (3.92–4.04) | 4.27× (4.24–4.30) | 4.68× (4.65–4.71) | 3.16× (3.13–3.18) | 2.90× (2.88–2.92) | 1.97× (1.94–2.00) | 2.08× (1.93–2.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,016 | 7.6 | 2.31× (2.12–2.52) | 3.13× (2.88–3.40) | 4.24× (3.89–4.62) | 3.82× (3.46–4.22) | 5.05× (4.67–5.47) | 4.89× (4.54–5.26) | 4.49× (4.17–4.83) | 3.05× (2.85–3.26) | 3.22× (2.85–3.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 995 | 14.0 | 1.90× (1.80–2.00) | 2.04× (1.91–2.18) | 3.04× (2.87–3.22) | 2.49× (2.36–2.62) | 4.26× (4.06–4.47) | 4.95× (4.72–5.19) | 4.55× (4.33–4.77) | 3.09× (2.94–3.24) | 3.27× (3.00–3.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.115 s | 70 | 56.5 | 1.35× (1.34–1.35) | 2.96× (2.93–2.98) | 2.68× (2.66–2.70) | 2.76× (2.75–2.78) | 3.07× (3.06–3.08) | 3.37× (3.35–3.39) | 3.50× (3.48–3.52) | 1.25× (1.24–1.26) | 1.24× (1.23–1.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 s | 117 | 61.6 | 1.29× (1.26–1.32) | 2.88× (2.78–2.97) | 2.79× (2.69–2.90) | 3.42× (3.36–3.48) | 3.19× (3.12–3.26) | 5.63× (5.51–5.76) | 5.85× (5.73–5.98) | 2.09× (2.04–2.14) | 2.07× (2.03–2.12) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 84.1 | 1.39× (1.36–1.42) | 2.79× (2.70–2.87) | 2.61× (2.53–2.69) | 3.25× (3.19–3.31) | 3.24× (3.17–3.31) | 6.89× (6.75–7.02) | 7.15× (7.00–7.30) | 2.55× (2.49–2.61) | 2.53× (2.48–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.126 s | 76 | 51.7 | 1.40× (1.39–1.41) | 3.33× (3.29–3.36) | 2.85× (2.81–2.89) | 2.93× (2.91–2.94) | 3.30× (3.28–3.32) | 4.03× (4.00–4.05) | 4.31× (4.29–4.34) | 1.46× (1.45–1.47) | 1.52× (1.51–1.53) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.074 s | 130 | 56.6 | 1.33× (1.30–1.36) | 3.15× (3.09–3.21) | 2.91× (2.85–2.96) | 3.88× (3.80–3.96) | 3.37× (3.30–3.44) | 6.91× (6.77–7.04) | 7.40× (7.26–7.54) | 2.51× (2.46–2.56) | 2.61× (2.56–2.66) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 149 | 80.6 | 1.40× (1.35–1.45) | 2.92× (2.84–3.00) | 2.68× (2.56–2.80) | 3.56× (3.47–3.65) | 3.17× (3.08–3.25) | 8.03× (7.82–8.24) | 8.60× (8.38–8.83) | 2.92× (2.84–2.99) | 3.03× (2.95–3.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 939 | 4.8 | 0.69× (0.67–0.70) | 2.91× (2.87–2.95) | 4.00× (3.94–4.06) | 3.25× (3.21–3.30) | 7.03× (6.94–7.11) | 2.49× (2.46–2.51) | 2.52× (2.50–2.55) | 1.04× (1.03–1.06) | 0.99× (0.98–1.01) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 866 | 5.3 | 0.52× (0.51–0.53) | 2.67× (2.62–2.72) | 5.44× (5.34–5.54) | 3.02× (2.96–3.08) | 6.52× (6.39–6.67) | 2.29× (2.24–2.33) | 2.32× (2.28–2.37) | 0.96× (0.93–0.99) | 0.91× (0.89–0.93) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 866 | 5.1 | 0.53× (0.52–0.54) | 2.70× (2.67–2.73) | 5.38× (5.15–5.62) | 3.08× (3.04–3.12) | 6.50× (6.42–6.59) | 2.30× (2.28–2.33) | 2.34× (2.31–2.36) | 0.97× (0.94–0.99) | 0.92× (0.91–0.93) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.060 s | 218 | 18.3 | 1.84× (1.83–1.85) | 4.22× (4.21–4.24) | 4.75× (4.73–4.76) | 4.75× (4.74–4.77) | 4.88× (4.86–4.90) | 5.13× (5.10–5.15) | 5.01× (4.99–5.03) | 1.50× (1.50–1.50) | 1.52× (1.52–1.53) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.039 s | 337 | 21.8 | 1.61× (1.46–1.77) | 3.58× (3.36–3.81) | 4.47× (4.10–4.88) | 4.54× (4.19–4.93) | 4.58× (4.24–4.95) | 7.69× (7.12–8.32) | 7.53× (6.96–8.14) | 2.25× (2.08–2.43) | 2.29× (2.12–2.47) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.033 s | 393 | 32.3 | 1.65× (1.57–1.72) | 3.12× (3.02–3.23) | 4.05× (3.92–4.18) | 4.13× (4.02–4.25) | 4.47× (4.34–4.61) | 9.16× (8.88–9.46) | 8.96× (8.70–9.24) | 2.68× (2.60–2.76) | 2.72× (2.64–2.81) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.40× (3.39–3.42) | 7.81× (7.76–7.86) | 8.32× (8.28–8.37) | 9.22× (9.15–9.29) | 8.39× (8.33–8.45) | 8.78× (8.73–8.82) | 8.49× (8.43–8.54) | 2.68× (2.67–2.69) | 2.37× (2.36–2.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.24× (3.21–3.26) | 7.56× (7.47–7.65) | 8.26× (8.21–8.31) | 9.01× (8.96–9.06) | 7.81× (7.75–7.88) | 8.09× (8.02–8.16) | 7.82× (7.75–7.89) | 2.47× (2.45–2.49) | 2.19× (2.17–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.34× (3.28–3.39) | 6.78× (6.68–6.89) | 7.24× (7.14–7.34) | 7.91× (7.80–8.01) | 7.06× (6.97–7.16) | 5.17× (5.09–5.25) | 5.00× (4.93–5.07) | 1.58× (1.56–1.60) | 1.40× (1.38–1.41) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.68× (0.64–0.72) | 2.96× (2.80–3.13) | 4.00× (3.78–4.23) | 3.59× (3.41–3.79) | 6.04× (5.72–6.38) | 2.74× (2.58–2.90) | 2.76× (2.61–2.93) | 1.12× (1.09–1.16) | 0.84× (0.80–0.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.60–0.64) | 2.93× (2.80–3.06) | 5.63× (5.34–5.94) | 3.49× (3.37–3.62) | 5.86× (5.65–6.08) | 2.65× (2.54–2.77) | 2.68× (2.57–2.79) | 1.09× (1.06–1.11) | 0.81× (0.79–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.58–0.64) | 3.26× (3.10–3.42) | 5.55× (5.31–5.81) | 3.54× (3.39–3.71) | 5.89× (5.64–6.14) | 2.68× (2.55–2.80) | 2.70× (2.59–2.82) | 1.10× (1.06–1.13) | 0.82× (0.79–0.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.47× (2.44–2.50) | 3.07× (3.05–3.08) | 3.61× (3.58–3.63) | 4.11× (4.09–4.14) | 4.50× (4.48–4.53) | 3.06× (3.03–3.08) | 2.81× (2.79–2.83) | 1.93× (1.89–1.96) | 1.94× (1.81–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.32× (2.23–2.42) | 2.82× (2.71–2.94) | 3.40× (3.30–3.50) | 3.60× (3.48–3.71) | 3.92× (3.80–4.05) | 2.59× (2.51–2.67) | 2.38× (2.30–2.46) | 1.63× (1.59–1.68) | 1.65× (1.52–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.06× (1.98–2.16) | 1.89× (1.80–1.99) | 2.37× (2.25–2.49) | 2.31× (2.19–2.43) | 2.42× (2.30–2.55) | 1.46× (1.39–1.53) | 1.34× (1.28–1.41) | 0.92× (0.88–0.96) | 0.93× (0.87–0.99) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.34–1.35) | 2.90× (2.89–2.91) | 2.63× (2.61–2.65) | 2.74× (2.72–2.75) | 3.04× (3.02–3.06) | 3.34× (3.32–3.36) | 3.47× (3.45–3.49) | 1.25× (1.24–1.25) | 1.21× (1.21–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.81× (2.78–2.84) | 2.55× (2.52–2.59) | 3.25× (3.21–3.28) | 2.85× (2.81–2.89) | 3.04× (3.01–3.08) | 3.16× (3.13–3.19) | 1.13× (1.12–1.15) | 1.11× (1.09–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.54× (1.51–1.56) | 2.81× (2.69–2.94) | 2.43× (2.35–2.52) | 3.19× (3.14–3.25) | 2.86× (2.81–2.92) | 2.25× (2.21–2.29) | 2.34× (2.29–2.38) | 0.84× (0.82–0.85) | 0.82× (0.80–0.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.39× (1.39–1.40) | 3.26× (3.23–3.28) | 2.80× (2.77–2.83) | 2.90× (2.88–2.92) | 3.27× (3.25–3.29) | 3.98× (3.96–4.00) | 4.27× (4.24–4.29) | 1.46× (1.45–1.46) | 1.45× (1.44–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.33–1.34) | 3.15× (3.13–3.17) | 2.69× (2.67–2.71) | 3.64× (3.62–3.65) | 3.08× (3.06–3.10) | 3.65× (3.63–3.67) | 3.91× (3.89–3.93) | 1.33× (1.33–1.34) | 1.33× (1.32–1.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.51–1.64) | 3.16× (3.06–3.27) | 2.41× (2.31–2.52) | 3.36× (3.25–3.46) | 2.91× (2.82–3.01) | 2.56× (2.47–2.64) | 2.74× (2.65–2.82) | 0.93× (0.91–0.96) | 0.93× (0.90–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.72–0.75) | 2.52× (2.47–2.58) | 3.50× (3.43–3.57) | 2.94× (2.89–2.99) | 6.20× (6.10–6.31) | 2.29× (2.25–2.32) | 2.32× (2.29–2.34) | 1.03× (1.02–1.05) | 0.77× (0.76–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.61) | 2.24× (2.20–2.28) | 4.30× (4.23–4.38) | 2.75× (2.69–2.81) | 5.70× (5.60–5.81) | 2.07× (2.03–2.11) | 2.10× (2.06–2.14) | 0.94× (0.90–0.97) | 0.70× (0.69–0.71) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.59) | 2.61× (2.52–2.71) | 4.45× (4.38–4.53) | 2.89× (2.86–2.92) | 5.88× (5.82–5.94) | 2.16× (2.13–2.18) | 2.19× (2.17–2.21) | 0.98× (0.95–1.00) | 0.73× (0.72–0.74) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.82× (1.81–1.83) | 4.04× (4.02–4.06) | 4.51× (4.49–4.53) | 4.65× (4.63–4.67) | 4.78× (4.75–4.80) | 5.01× (4.99–5.03) | 4.90× (4.88–4.92) | 1.49× (1.48–1.49) | 1.38× (1.37–1.39) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.62× (1.61–1.64) | 3.58× (3.53–3.64) | 4.20× (4.17–4.22) | 4.47× (4.43–4.51) | 4.16× (4.11–4.20) | 4.22× (4.19–4.25) | 4.13× (4.11–4.15) | 1.25× (1.24–1.26) | 1.16× (1.15–1.17) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.76× (1.67–1.86) | 3.25× (3.17–3.33) | 3.63× (3.54–3.73) | 4.03× (3.94–4.11) | 3.67× (3.62–3.73) | 2.84× (2.80–2.88) | 2.78× (2.74–2.82) | 0.84× (0.83–0.86) | 0.78× (0.77–0.79) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.27× (0.26–0.28) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.00× (10.96–11.04) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.50× (0.42–0.60) | 0.69× (0.66–0.71) | 0.58× (0.56–0.60) | 5.47× (5.29–5.65) | 0.13× (0.13–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.34× (0.32–0.35) | 0.49× (0.44–0.55) | 0.60× (0.58–0.62) | 0.54× (0.51–0.56) | 3.52× (3.36–3.67) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.84× (0.84–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.41–0.45) | 1.71× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.02–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.67× (0.60–0.75) | 1.62× (1.61–1.63) | 0.81× (0.80–0.82) | 5.86× (5.81–5.90) | 0.47× (0.47–0.48) | 0.50× (0.49–0.50) | 0.54× (0.53–0.54) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.62–0.66) | 0.67× (0.62–0.71) | 1.14× (1.10–1.18) | 0.69× (0.67–0.73) | 3.38× (3.27–3.49) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.17× (5.17–5.18) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.47–0.51) | 0.39× (0.37–0.42) | 0.89× (0.85–0.92) | 0.68× (0.65–0.72) | 3.06× (2.93–3.19) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.46× (0.44–0.47) | 0.87× (0.85–0.89) | 0.82× (0.80–0.83) | 2.52× (2.47–2.58) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.17–5.20) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.50) | 0.38× (0.35–0.41) | 0.84× (0.81–0.88) | 0.71× (0.67–0.75) | 2.90× (2.78–3.02) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.51× (0.48–0.54) | 0.81× (0.80–0.83) | 0.83× (0.81–0.85) | 2.35× (2.30–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.07× (1.07–1.07) | 0.77× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.30) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.42) | 0.67× (0.66–0.67) | 6.44× (6.43–6.45) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 38 MB | 0.45× (0.43–0.46) | 0.39× (0.35–0.42) | 1.04× (1.00–1.07) | 0.69× (0.67–0.72) | 3.32× (3.23–3.41) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 57 MB | 0.48× (0.47–0.49) | 0.47× (0.43–0.50) | 0.89× (0.88–0.90) | 0.73× (0.71–0.76) | 2.29× (2.26–2.33) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 537 | 1.87 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 949 | 2.03 | 1.09× (1.08–1.10) | 87% | 1.05× (1.04–1.06) | 1.03× (1.03–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,133 | 3.18 | 1.70× (1.68–1.72) | 52% | 1.47× (1.46–1.49) | 1.66× (1.65–1.68) |

**Best:** mibz2 at 4 threads (1,133 MB/s); lbzip2 at 4 threads (181 MB/s); pbz2 at 4 threads (336 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.90, mibz2 4t 1.24, lbzip2 1t 0.04, lbzip2 2t 0.29, lbzip2 4t 0.34, pbz2 1t 0.00, pbz2 2t 0.27, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.49 | 0.02 | 0.08 | 0.185 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3882 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 55.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 10.8% |
| MTF + RLE2 | 1.66 | 15.7% |
| T-vector build | 1.51 | 14.2% |
| walk (the one each build uses) | 5.91 | 55.7% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.24 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.60 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.367 | 0.930 | (in its total) | 2.53× (2.53–2.54) |  |
| walk | 1.437 | 2.747 | (in its total) | 1.91× (1.91–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.072 | (in its total) | 0.91× (0.91–0.91) |  |
| inverse BWT, total | 1.884 | 3.749 | 1.970 | 1.99× (1.99–1.99) | 1.05× (1.04–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m50s ± 6m29s |
| 4 threads | 27m04s ± 3m49s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m07s ± 2m34s |
| the scanner alone (its serial part) | 10m37s ± 30.2s |
| the workers alone, over 4 | 18m07s ± 2m34s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 2.38 ns a byte of output at 4 threads (CPU, summed over them), 1.51 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3882.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261116.88.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
