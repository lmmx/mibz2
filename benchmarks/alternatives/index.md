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

The latest record: [`34132f8`](<https://github.com/lmmx/mibz2/commit/34132f890ab588ffd03f16b99ad5b12e2f613875>) (2026-11-24).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45400965133>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.385 s | 545 | 7.2 | 3.58× (3.57–3.59) | 8.18× (8.14–8.22) | 8.73× (8.69–8.77) | 9.64× (9.61–9.68) | 8.73× (8.68–8.78) | 9.15× (9.09–9.21) | 8.89× (8.83–8.95) | 2.83× (2.82–2.84) | 2.49× (2.48–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.227 s | 922 | 8.2 | 3.30× (3.23–3.36) | 7.20× (6.99–7.42) | 8.66× (8.47–8.86) | 8.57× (8.41–8.72) | 7.70× (7.56–7.84) | 15.45× (15.14–15.76) | 15.01× (14.71–15.33) | 4.78× (4.70–4.86) | 4.21× (4.14–4.29) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.186 s | 1,127 | 12.9 | 3.42× (3.39–3.46) | 6.28× (6.21–6.36) | 7.45× (7.35–7.54) | 7.34× (7.30–7.38) | 7.14× (7.07–7.21) | 18.91× (18.66–19.16) | 18.38× (18.14–18.63) | 5.85× (5.80–5.91) | 5.16× (5.10–5.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,231 | 3.5 | 0.61× (0.60–0.63) | 3.09× (3.02–3.17) | 4.41× (4.30–4.53) | 3.69× (3.61–3.77) | 6.31× (6.18–6.45) | 2.79× (2.73–2.85) | 2.82× (2.76–2.89) | 1.15× (1.12–1.18) | 1.25× (1.22–1.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,156 | 3.8 | 0.52× (0.52–0.53) | 2.93× (2.88–2.99) | 5.91× (5.80–6.03) | 3.52× (3.47–3.57) | 5.95× (5.87–6.02) | 2.64× (2.60–2.68) | 2.67× (2.63–2.71) | 1.09× (1.06–1.12) | 1.18× (1.16–1.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,156 | 3.8 | 0.53× (0.52–0.54) | 2.95× (2.87–3.02) | 5.82× (5.59–6.04) | 3.52× (3.44–3.61) | 5.94× (5.82–6.05) | 2.65× (2.59–2.70) | 2.68× (2.62–2.74) | 1.09× (1.05–1.13) | 1.19× (1.16–1.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 633 | 6.5 | 2.61× (2.55–2.67) | 3.28× (3.25–3.32) | 3.95× (3.86–4.04) | 4.25× (4.20–4.29) | 4.69× (4.64–4.74) | 3.14× (3.10–3.18) | 2.90× (2.87–2.93) | 2.07× (2.04–2.11) | 2.16× (2.05–2.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,016 | 7.6 | 2.33× (2.18–2.48) | 3.12× (2.92–3.33) | 4.23× (3.96–4.52) | 3.91× (3.63–4.22) | 5.21× (4.91–5.52) | 4.95× (4.64–5.28) | 4.58× (4.28–4.89) | 3.26× (3.03–3.51) | 3.41× (3.07–3.78) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,071 | 13.6 | 2.10× (2.03–2.16) | 2.26× (2.20–2.32) | 3.26× (3.06–3.47) | 2.61× (2.52–2.70) | 4.59× (4.46–4.72) | 5.23× (5.07–5.39) | 4.83× (4.69–4.98) | 3.45× (3.37–3.53) | 3.60× (3.38–3.83) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 55.9 | 1.39× (1.36–1.42) | 3.06× (3.00–3.13) | 2.81× (2.74–2.88) | 2.88× (2.83–2.93) | 3.17× (3.10–3.23) | 3.49× (3.42–3.55) | 3.56× (3.49–3.62) | 1.27× (1.25–1.30) | 1.25× (1.23–1.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.063 s | 126 | 59.5 | 1.41× (1.39–1.43) | 3.09× (3.03–3.15) | 3.02× (2.97–3.07) | 3.72× (3.61–3.83) | 3.44× (3.38–3.50) | 6.20× (6.09–6.31) | 6.32× (6.20–6.44) | 2.26× (2.22–2.31) | 2.23× (2.19–2.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 154 | 79.8 | 1.52× (1.49–1.56) | 2.96× (2.89–3.04) | 2.86× (2.75–2.97) | 3.51× (3.41–3.61) | 3.47× (3.37–3.58) | 7.59× (7.37–7.80) | 7.74× (7.52–7.96) | 2.77× (2.69–2.85) | 2.72× (2.65–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.124 s | 77 | 51.1 | 1.43× (1.37–1.49) | 3.45× (3.31–3.60) | 2.91× (2.79–3.04) | 3.06× (2.94–3.19) | 3.39× (3.25–3.54) | 4.12× (3.95–4.29) | 4.37× (4.19–4.56) | 1.48× (1.42–1.54) | 1.53× (1.46–1.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 137 | 54.9 | 1.37× (1.31–1.44) | 3.30× (3.18–3.42) | 2.99× (2.86–3.12) | 4.06× (3.87–4.26) | 3.45× (3.31–3.61) | 7.25× (6.94–7.57) | 7.69× (7.35–8.04) | 2.60× (2.49–2.72) | 2.69× (2.57–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.058 s | 165 | 78.2 | 1.57× (1.53–1.61) | 3.18× (3.10–3.27) | 2.98× (2.90–3.06) | 3.94× (3.85–4.03) | 3.50× (3.43–3.57) | 8.92× (8.74–9.10) | 9.46× (9.25–9.67) | 3.20× (3.14–3.27) | 3.30× (3.24–3.38) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 926 | 4.9 | 0.70× (0.68–0.72) | 2.84× (2.79–2.89) | 4.09× (3.83–4.36) | 3.21× (3.15–3.27) | 7.08× (6.71–7.47) | 2.45× (2.41–2.49) | 2.48× (2.45–2.52) | 1.06× (1.04–1.08) | 0.97× (0.95–0.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 844 | 5.4 | 0.51× (0.50–0.53) | 2.60× (2.54–2.65) | 5.27× (5.16–5.40) | 2.93× (2.87–3.00) | 6.31× (6.15–6.48) | 2.24× (2.20–2.28) | 2.27× (2.22–2.32) | 0.97× (0.94–0.99) | 0.88× (0.87–0.90) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 848 | 5.4 | 0.52× (0.51–0.52) | 2.62× (2.59–2.64) | 5.31× (5.27–5.36) | 2.98× (2.96–3.01) | 6.34× (6.30–6.39) | 2.25× (2.23–2.27) | 2.28× (2.26–2.30) | 0.97× (0.96–0.98) | 0.89× (0.88–0.90) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 0.062 s | 226 | 17.7 | 1.94× (1.93–1.95) | 4.44× (4.42–4.46) | 4.95× (4.92–4.97) | 5.03× (5.01–5.06) | 5.11× (5.09–5.13) | 5.40× (5.38–5.42) | 5.28× (5.25–5.31) | 1.59× (1.58–1.59) | 1.48× (1.47–1.49) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 0.038 s | 363 | 20.8 | 1.65× (1.59–1.72) | 3.75× (3.60–3.90) | 4.98× (4.76–5.21) | 5.06× (4.83–5.29) | 4.78× (4.56–5.01) | 8.53× (8.16–8.93) | 8.34× (7.96–8.73) | 2.51× (2.40–2.62) | 2.34× (2.24–2.45) | yes |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 0.033 s | 421 | 30.8 | 1.72× (1.66–1.78) | 3.32× (3.19–3.47) | 4.44× (4.34–4.54) | 4.57× (4.45–4.69) | 4.77× (4.66–4.88) | 10.07× (9.79–10.36) | 9.84× (9.56–10.13) | 2.96× (2.88–3.04) | 2.76× (2.69–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.57× (3.55–3.58) | 8.13× (8.09–8.17) | 8.69× (8.64–8.73) | 9.61× (9.57–9.64) | 8.69× (8.64–8.74) | 9.11× (9.06–9.17) | 8.86× (8.80–8.92) | 2.82× (2.81–2.83) | 2.47× (2.46–2.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.16× (3.14–3.18) | 7.31× (7.24–7.38) | 8.04× (7.99–8.09) | 8.76× (8.71–8.81) | 7.55× (7.50–7.59) | 7.96× (7.89–8.02) | 7.73× (7.66–7.80) | 2.46× (2.45–2.48) | 2.16× (2.14–2.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.25× (3.20–3.30) | 6.60× (6.50–6.69) | 7.08× (6.99–7.18) | 7.71× (7.60–7.82) | 6.87× (6.77–6.98) | 5.08× (5.00–5.16) | 4.93× (4.86–5.00) | 1.57× (1.55–1.60) | 1.38× (1.36–1.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.67) | 2.85× (2.79–2.91) | 3.86× (3.78–3.93) | 3.44× (3.38–3.51) | 5.83× (5.71–5.95) | 2.62× (2.57–2.68) | 2.66× (2.61–2.72) | 1.14× (1.11–1.16) | 0.82× (0.79–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.65× (2.61–2.70) | 5.16× (5.07–5.24) | 3.21× (3.16–3.26) | 5.35× (5.28–5.42) | 2.41× (2.38–2.45) | 2.45× (2.41–2.48) | 1.05× (1.02–1.08) | 0.75× (0.73–0.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.57) | 3.08× (3.01–3.16) | 5.13× (4.99–5.26) | 3.26× (3.19–3.34) | 5.40× (5.30–5.50) | 2.46× (2.41–2.51) | 2.49× (2.44–2.54) | 1.06× (1.03–1.10) | 0.77× (0.74–0.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.52× (2.47–2.58) | 3.04× (3.01–3.07) | 3.56× (3.52–3.59) | 4.08× (4.04–4.12) | 4.50× (4.45–4.55) | 3.03× (2.99–3.07) | 2.80× (2.78–2.83) | 2.01× (1.98–2.05) | 1.92× (1.89–1.94) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.37× (2.32–2.42) | 2.84× (2.76–2.92) | 3.39× (3.29–3.48) | 3.60× (3.50–3.69) | 3.96× (3.87–4.06) | 2.59× (2.53–2.65) | 2.40× (2.33–2.46) | 1.72× (1.67–1.78) | 1.64× (1.58–1.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.05× (1.96–2.15) | 1.91× (1.86–1.95) | 2.32× (2.28–2.37) | 2.29× (2.24–2.33) | 2.43× (2.38–2.49) | 1.44× (1.41–1.48) | 1.34× (1.31–1.36) | 0.96× (0.95–0.97) | 0.91× (0.89–0.93) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.41) | 3.01× (2.95–3.06) | 2.75× (2.69–2.81) | 2.85× (2.80–2.90) | 3.13× (3.07–3.20) | 3.45× (3.39–3.51) | 3.52× (3.45–3.58) | 1.27× (1.25–1.29) | 1.24× (1.21–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.35× (1.33–1.37) | 2.95× (2.90–2.99) | 2.68× (2.64–2.71) | 3.41× (3.36–3.45) | 2.98× (2.94–3.02) | 3.24× (3.19–3.28) | 3.30× (3.25–3.35) | 1.19× (1.18–1.21) | 1.16× (1.15–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.64× (1.61–1.66) | 3.09× (3.03–3.14) | 2.65× (2.60–2.71) | 3.40× (3.34–3.46) | 3.03× (2.98–3.08) | 2.42× (2.38–2.47) | 2.47× (2.42–2.52) | 0.89× (0.88–0.91) | 0.87× (0.85–0.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.42× (1.36–1.48) | 3.37× (3.23–3.51) | 2.86× (2.74–2.98) | 3.03× (2.91–3.16) | 3.36× (3.22–3.50) | 4.07× (3.91–4.24) | 4.32× (4.14–4.51) | 1.47× (1.41–1.53) | 1.46× (1.40–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.36–1.41) | 3.26× (3.19–3.32) | 2.80× (2.75–2.85) | 3.77× (3.70–3.84) | 3.17× (3.11–3.23) | 3.83× (3.76–3.90) | 4.06× (3.98–4.14) | 1.38× (1.36–1.41) | 1.37× (1.35–1.39) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.63–1.71) | 3.30× (3.23–3.36) | 2.57× (2.51–2.63) | 3.55× (3.50–3.61) | 3.07× (3.03–3.11) | 2.71× (2.68–2.75) | 2.88× (2.84–2.92) | 0.98× (0.97–0.99) | 0.97× (0.96–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.72–0.75) | 2.52× (2.49–2.56) | 3.50× (3.43–3.56) | 2.91× (2.87–2.96) | 6.27× (5.94–6.62) | 2.26× (2.22–2.30) | 2.29× (2.26–2.32) | 1.05× (1.04–1.07) | 0.76× (0.75–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.55–0.59) | 2.16× (2.12–2.21) | 4.19× (4.11–4.28) | 2.65× (2.60–2.71) | 5.49× (5.34–5.64) | 2.01× (1.97–2.06) | 2.04× (2.00–2.08) | 0.94× (0.92–0.96) | 0.68× (0.66–0.70) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.45× (2.36–2.53) | 4.25× (4.19–4.30) | 2.74× (2.71–2.77) | 5.62× (5.56–5.68) | 2.06× (2.05–2.08) | 2.09× (2.07–2.11) | 0.96× (0.95–0.97) | 0.70× (0.69–0.71) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 1.92× (1.91–1.93) | 4.27× (4.25–4.30) | 4.78× (4.76–4.80) | 4.93× (4.90–4.95) | 5.01× (4.99–5.02) | 5.28× (5.26–5.31) | 5.16× (5.13–5.19) | 1.57× (1.57–1.58) | 1.45× (1.44–1.46) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 1.69× (1.67–1.71) | 3.77× (3.72–3.82) | 4.40× (4.35–4.45) | 4.66× (4.62–4.71) | 4.29× (4.24–4.34) | 4.47× (4.41–4.53) | 4.37× (4.30–4.44) | 1.33× (1.31–1.35) | 1.23× (1.21–1.24) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 1.87× (1.82–1.92) | 3.47× (3.40–3.54) | 3.84× (3.76–3.92) | 4.21× (4.12–4.29) | 3.89× (3.81–3.96) | 3.03× (2.96–3.09) | 2.96× (2.89–3.03) | 0.90× (0.88–0.92) | 0.83× (0.81–0.85) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.26–0.29) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.00× (10.95–11.04) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.55× (0.50–0.61) | 0.73× (0.70–0.75) | 0.61× (0.59–0.64) | 5.76× (5.58–5.95) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.36–0.39) | 0.50× (0.43–0.58) | 0.66× (0.63–0.69) | 0.57× (0.55–0.60) | 3.85× (3.69–4.02) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.27) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.17) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.84× (4.83–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.44) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.96× (7.91–8.01) | 0.65× (0.65–0.66) | 0.68× (0.68–0.68) | 0.73× (0.72–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.72× (0.70–0.74) | 0.75× (0.66–0.85) | 1.72× (1.67–1.77) | 0.86× (0.83–0.88) | 6.23× (6.05–6.41) | 0.51× (0.49–0.52) | 0.53× (0.51–0.55) | 0.57× (0.56–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.71× (0.69–0.73) | 0.77× (0.73–0.81) | 1.26× (1.24–1.28) | 0.78× (0.77–0.79) | 3.75× (3.68–3.82) | 0.30× (0.29–0.30) | 0.31× (0.30–0.31) | 0.33× (0.33–0.34) | 0.17× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.17) | 0.73× (0.72–0.73) | 5.14× (5.12–5.16) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.52) | 0.42× (0.39–0.44) | 0.92× (0.90–0.94) | 0.72× (0.70–0.75) | 3.17× (3.09–3.25) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.47× (0.46–0.47) | 0.88× (0.86–0.90) | 0.81× (0.78–0.85) | 2.56× (2.51–2.62) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.17–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.36× (0.33–0.40) | 0.85× (0.81–0.88) | 0.69× (0.65–0.73) | 2.88× (2.77–3.00) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.62× (0.61–0.63) | 0.47× (0.45–0.48) | 0.81× (0.80–0.82) | 0.82× (0.79–0.84) | 2.33× (2.31–2.36) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.06× (1.06–1.06) | 0.76× (0.75–0.76) | 5.30× (5.28–5.31) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.21× (6.20–6.22) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.18× (6.16–6.21) | 0.40× (0.40–0.40) | 0.43× (0.43–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.66–0.67) | 6.40× (6.39–6.42) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 2 | 36 MB | 0.46× (0.45–0.48) | 0.41× (0.38–0.45) | 1.06× (1.02–1.09) | 0.71× (0.69–0.73) | 3.44× (3.34–3.55) | 0.21× (0.20–0.22) | 0.22× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.75 MB → 13.9 MB, 5.1×) | 4 | 54 MB | 0.50× (0.48–0.52) | 0.49× (0.44–0.55) | 0.94× (0.92–0.96) | 0.78× (0.76–0.81) | 2.39× (2.33–2.45) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 545 | 1.84 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 922 | 2.11 | 1.15× (1.14–1.15) | 84% | 1.03× (1.02–1.04) | 1.01× (1.01–1.02) |
| 4: all 4 CPUs, sharing SMT siblings | 1,127 | 3.31 | 1.79× (1.77–1.82) | 52% | 1.46× (1.45–1.46) | 1.63× (1.62–1.65) |

**Best:** mibz2 at 4 threads (1,127 MB/s); lbzip2 at 4 threads (179 MB/s); pbz2 at 4 threads (329 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.03, lbzip2 1t 0.04, lbzip2 2t 0.22, lbzip2 4t 0.28, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.53 | 0.02 | 0.09 | 0.186 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3900 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 57.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.97 | 8.9% |
| MTF + RLE2 | 1.61 | 14.9% |
| T-vector build | 1.57 | 14.5% |
| walk (the one each build uses) | 6.27 | 57.9% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.25 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.83 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.372 | 0.930 | (in its total) | 2.50× (2.50–2.50) |  |
| walk | 1.512 | 2.908 | (in its total) | 1.92× (1.92–1.93) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.074 | (in its total) | 0.91× (0.91–0.91) |  |
| inverse BWT, total | 1.965 | 3.912 | 2.017 | 1.99× (1.99–2.00) | 1.03× (1.03–1.03) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 55m59s ± 6m37s |
| 4 threads | 27m05s ± 3m48s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m12s ± 2m45s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 19m12s ± 2m45s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.52 ns a byte of output at 4 threads (CPU, summed over them), 1.55 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,754,972 | 13,944,546 | \`69da7dbeba859e79…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3900.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261116.88.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
