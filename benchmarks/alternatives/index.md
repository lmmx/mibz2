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

The latest record: [`7e3b601`](<https://github.com/lmmx/mibz2/commit/7e3b601a0d41132464be3f76546120de67037888>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44846170376>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.389 s | 539 | 7.3 | 3.51× (3.50–3.52) | 7.95× (7.92–7.99) | 8.48× (8.45–8.51) | 9.38× (9.33–9.43) | 8.52× (8.49–8.55) | 8.89× (8.87–8.91) | 8.69× (8.68–8.70) | 2.73× (2.72–2.74) | 2.42× (2.41–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.222 s | 946 | 7.9 | 3.36× (3.18–3.55) | 7.24× (6.89–7.60) | 8.67× (8.20–9.17) | 8.57× (8.12–9.06) | 7.80× (7.39–8.23) | 15.19× (14.38–16.05) | 14.85× (14.07–15.66) | 4.66× (4.42–4.92) | 4.14× (3.92–4.37) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.183 s | 1,146 | 12.4 | 3.52× (3.48–3.56) | 6.46× (6.34–6.59) | 7.51× (7.39–7.64) | 7.44× (7.34–7.53) | 7.29× (7.20–7.38) | 18.83× (18.58–19.09) | 18.40× (18.16–18.65) | 5.78× (5.71–5.85) | 5.14× (5.08–5.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,282 | 3.3 | 0.62× (0.61–0.64) | 3.26× (3.20–3.33) | 4.67× (4.58–4.76) | 3.82× (3.75–3.89) | 6.46× (6.34–6.57) | 2.91× (2.86–2.97) | 2.94× (2.89–2.99) | 1.10× (1.05–1.14) | 1.32× (1.29–1.35) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,258 | 3.5 | 0.58× (0.57–0.59) | 3.22× (3.14–3.29) | 6.45× (6.25–6.65) | 3.78× (3.69–3.86) | 6.36× (6.22–6.50) | 2.87× (2.80–2.93) | 2.89× (2.83–2.96) | 1.08× (1.02–1.14) | 1.30× (1.27–1.33) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,246 | 3.4 | 0.58× (0.56–0.61) | 3.22× (3.09–3.35) | 6.31× (5.92–6.72) | 3.82× (3.65–4.00) | 6.39× (6.15–6.64) | 2.86× (2.76–2.97) | 2.89× (2.78–3.00) | 1.08× (1.02–1.14) | 1.30× (1.25–1.35) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 649 | 6.3 | 2.93× (2.82–3.04) | 3.42× (3.28–3.56) | 3.98× (3.88–4.09) | 4.40× (4.23–4.58) | 4.82× (4.62–5.02) | 3.24× (3.11–3.37) | 2.99× (2.88–3.11) | 2.01× (1.94–2.07) | 2.12× (2.01–2.23) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,021 | 7.6 | 2.61× (2.42–2.82) | 3.19× (3.05–3.34) | 4.34× (4.14–4.54) | 3.94× (3.71–4.19) | 5.26× (4.99–5.54) | 5.01× (4.80–5.24) | 4.64× (4.42–4.86) | 3.11× (2.95–3.27) | 3.28× (3.07–3.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,045 | 13.3 | 2.16× (2.08–2.24) | 2.28× (2.20–2.36) | 3.18× (3.00–3.38) | 2.75× (2.54–2.97) | 4.50× (4.33–4.68) | 5.21× (5.02–5.41) | 4.82× (4.65–5.00) | 3.23× (3.11–3.35) | 3.41× (3.25–3.57) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.113 s | 71 | 56.0 | 1.38× (1.37–1.38) | 3.01× (2.98–3.05) | 2.77× (2.74–2.80) | 2.83× (2.82–2.85) | 3.09× (3.07–3.10) | 3.44× (3.43–3.45) | 3.54× (3.53–3.56) | 1.25× (1.24–1.25) | 1.26× (1.25–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 119 | 61.0 | 1.33× (1.27–1.40) | 2.86× (2.81–2.92) | 2.83× (2.74–2.92) | 3.48× (3.39–3.57) | 3.19× (3.11–3.28) | 5.72× (5.57–5.87) | 5.88× (5.74–6.03) | 2.07× (2.02–2.12) | 2.08× (2.03–2.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 82.3 | 1.41× (1.37–1.46) | 2.84× (2.76–2.92) | 2.64× (2.58–2.70) | 3.29× (3.19–3.40) | 3.22× (3.14–3.30) | 7.01× (6.84–7.19) | 7.22× (7.05–7.39) | 2.54× (2.48–2.60) | 2.56× (2.49–2.62) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.125 s | 77 | 51.1 | 1.44× (1.44–1.45) | 3.42× (3.39–3.46) | 2.90× (2.87–2.93) | 3.00× (2.98–3.01) | 3.34× (3.30–3.37) | 4.12× (4.11–4.13) | 4.44× (4.42–4.45) | 1.46× (1.46–1.47) | 1.55× (1.54–1.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 131 | 56.1 | 1.32× (1.25–1.39) | 3.11× (2.98–3.25) | 2.83× (2.70–2.98) | 3.84× (3.64–4.05) | 3.25× (3.10–3.42) | 6.80× (6.47–7.14) | 7.32× (6.97–7.69) | 2.42× (2.30–2.53) | 2.55× (2.43–2.68) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.061 s | 157 | 79.5 | 1.46× (1.42–1.51) | 3.00× (2.93–3.08) | 2.71× (2.61–2.81) | 3.71× (3.60–3.81) | 3.25× (3.17–3.32) | 8.31× (8.10–8.53) | 8.95× (8.72–9.19) | 2.95× (2.87–3.03) | 3.12× (3.04–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 962 | 4.7 | 0.69× (0.66–0.72) | 3.10× (2.87–3.33) | 4.31× (4.00–4.64) | 3.39× (3.17–3.62) | 7.28× (6.84–7.76) | 2.59× (2.41–2.78) | 2.63× (2.43–2.85) | 1.04× (0.98–1.11) | 1.06× (0.98–1.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 935 | 4.8 | 0.57× (0.55–0.59) | 2.91× (2.80–3.02) | 5.84× (5.54–6.17) | 3.24× (3.14–3.35) | 6.90× (6.65–7.17) | 2.43× (2.34–2.52) | 2.47× (2.36–2.58) | 0.98× (0.94–1.02) | 0.99× (0.96–1.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 922 | 4.8 | 0.57× (0.55–0.59) | 2.85× (2.78–2.93) | 5.52× (5.22–5.83) | 3.23× (3.13–3.33) | 6.84× (6.63–7.06) | 2.38× (2.31–2.44) | 2.42× (2.33–2.51) | 0.95× (0.93–0.98) | 0.97× (0.95–0.99) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 0.062 s | 215 | 18.6 | 1.83× (1.81–1.86) | 4.12× (4.08–4.17) | 4.62× (4.55–4.70) | 4.72× (4.65–4.79) | 4.84× (4.77–4.91) | 5.10× (5.02–5.18) | 5.06× (4.99–5.13) | 1.47× (1.45–1.50) | 1.48× (1.46–1.50) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 0.039 s | 337 | 21.5 | 1.62× (1.58–1.67) | 3.67× (3.53–3.81) | 4.56× (4.40–4.73) | 4.76× (4.59–4.93) | 4.72× (4.58–4.86) | 8.00× (7.78–8.24) | 7.94× (7.73–8.15) | 2.31× (2.25–2.38) | 2.32× (2.26–2.39) | yes |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 0.034 s | 389 | 31.5 | 1.66× (1.62–1.69) | 3.15× (2.99–3.32) | 3.92× (3.78–4.07) | 4.14× (4.01–4.28) | 4.45× (4.34–4.56) | 9.20× (8.99–9.42) | 9.13× (8.92–9.35) | 2.66× (2.60–2.72) | 2.67× (2.61–2.74) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.50× (3.49–3.51) | 7.91× (7.88–7.95) | 8.43× (8.40–8.47) | 9.35× (9.30–9.40) | 8.49× (8.46–8.52) | 8.86× (8.84–8.88) | 8.66× (8.64–8.67) | 2.72× (2.71–2.73) | 2.41× (2.40–2.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.35× (3.33–3.37) | 7.71× (7.61–7.81) | 8.43× (8.38–8.48) | 9.17× (9.12–9.23) | 7.99× (7.92–8.05) | 8.16× (8.10–8.22) | 7.98× (7.92–8.03) | 2.51× (2.49–2.53) | 2.22× (2.19–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.43× (3.37–3.50) | 7.00× (6.89–7.10) | 7.46× (7.33–7.59) | 8.15× (8.03–8.27) | 7.30× (7.19–7.42) | 5.25× (5.17–5.33) | 5.13× (5.05–5.21) | 1.61× (1.59–1.64) | 1.43× (1.40–1.45) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.66× (0.64–0.67) | 2.97× (2.90–3.04) | 3.99× (3.92–4.06) | 3.57× (3.50–3.63) | 5.96× (5.86–6.06) | 2.74× (2.68–2.80) | 2.76× (2.71–2.82) | 1.08× (1.04–1.13) | 0.83× (0.81–0.84) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.64) | 2.92× (2.85–3.00) | 5.57× (5.42–5.72) | 3.48× (3.39–3.56) | 5.77× (5.63–5.92) | 2.65× (2.58–2.73) | 2.68× (2.61–2.75) | 1.05× (0.99–1.11) | 0.80× (0.78–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.59–0.64) | 3.41× (3.30–3.52) | 5.63× (5.41–5.87) | 3.60× (3.44–3.77) | 5.93× (5.72–6.15) | 2.72× (2.62–2.82) | 2.74× (2.64–2.84) | 1.07× (1.02–1.13) | 0.82× (0.78–0.86) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.85× (2.73–2.96) | 3.17× (3.04–3.30) | 3.70× (3.54–3.86) | 4.26× (4.08–4.44) | 4.66× (4.45–4.87) | 3.14× (3.01–3.28) | 2.91× (2.79–3.03) | 1.97× (1.90–2.04) | 1.94× (1.88–2.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.52× (2.43–2.61) | 2.77× (2.68–2.87) | 3.36× (3.25–3.47) | 3.60× (3.47–3.75) | 3.95× (3.79–4.11) | 2.57× (2.49–2.66) | 2.38× (2.30–2.46) | 1.61× (1.55–1.67) | 1.59× (1.53–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.20× (2.06–2.34) | 1.94× (1.89–1.99) | 2.37× (2.26–2.47) | 2.33× (2.24–2.41) | 2.46× (2.37–2.56) | 1.47× (1.41–1.53) | 1.36× (1.31–1.41) | 0.92× (0.89–0.96) | 0.91× (0.88–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.94× (2.93–2.95) | 2.70× (2.68–2.72) | 2.81× (2.79–2.82) | 3.06× (3.04–3.07) | 3.41× (3.40–3.42) | 3.51× (3.50–3.52) | 1.24× (1.24–1.25) | 1.23× (1.23–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.30× (1.28–1.33) | 2.84× (2.79–2.90) | 2.59× (2.54–2.65) | 3.31× (3.24–3.37) | 2.88× (2.82–2.95) | 3.09× (3.02–3.16) | 3.18× (3.12–3.25) | 1.13× (1.10–1.15) | 1.11× (1.09–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.53–1.61) | 2.90× (2.79–3.02) | 2.54× (2.50–2.58) | 3.28× (3.22–3.34) | 2.92× (2.88–2.97) | 2.32× (2.28–2.36) | 2.39× (2.35–2.43) | 0.85× (0.83–0.86) | 0.84× (0.82–0.85) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.44) | 3.34× (3.31–3.37) | 2.84× (2.82–2.87) | 2.98× (2.96–2.99) | 3.31× (3.28–3.34) | 4.08× (4.07–4.10) | 4.40× (4.38–4.41) | 1.46× (1.45–1.46) | 1.47× (1.47–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.38) | 3.21× (3.19–3.24) | 2.75× (2.72–2.78) | 3.72× (3.68–3.75) | 3.10× (3.07–3.13) | 3.72× (3.68–3.76) | 4.00× (3.97–4.04) | 1.33× (1.31–1.34) | 1.34× (1.32–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.58–1.68) | 3.27× (3.18–3.37) | 2.52× (2.43–2.61) | 3.49× (3.38–3.61) | 3.00× (2.92–3.09) | 2.65× (2.57–2.72) | 2.85× (2.77–2.93) | 0.95× (0.92–0.97) | 0.95× (0.93–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.71–0.76) | 2.65× (2.48–2.83) | 3.65× (3.44–3.87) | 3.08× (2.89–3.29) | 6.49× (6.08–6.93) | 2.39× (2.23–2.57) | 2.42× (2.24–2.62) | 1.04× (0.98–1.11) | 0.80× (0.76–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.61–0.64) | 2.41× (2.33–2.49) | 4.63× (4.45–4.81) | 2.96× (2.87–3.04) | 6.08× (5.89–6.29) | 2.21× (2.14–2.28) | 2.23× (2.15–2.32) | 0.96× (0.93–0.99) | 0.74× (0.71–0.76) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.63) | 2.77× (2.64–2.92) | 4.60× (4.48–4.73) | 3.03× (2.94–3.12) | 6.21× (6.02–6.40) | 2.23× (2.16–2.29) | 2.25× (2.17–2.34) | 0.97× (0.94–1.00) | 0.74× (0.72–0.77) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 1.81× (1.79–1.84) | 4.00× (3.94–4.06) | 4.48× (4.42–4.54) | 4.63× (4.55–4.71) | 4.75× (4.68–4.83) | 5.00× (4.92–5.09) | 4.96× (4.88–5.04) | 1.46× (1.44–1.48) | 1.37× (1.34–1.39) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 1.66× (1.63–1.69) | 3.66× (3.57–3.75) | 4.31× (4.23–4.38) | 4.59× (4.51–4.68) | 4.25× (4.17–4.32) | 4.33× (4.25–4.42) | 4.30× (4.21–4.38) | 1.27× (1.24–1.29) | 1.18× (1.16–1.21) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 1.83× (1.76–1.91) | 3.36× (3.29–3.43) | 3.79× (3.71–3.88) | 4.16× (4.08–4.24) | 3.80× (3.73–3.87) | 2.94× (2.89–3.00) | 2.92× (2.87–2.98) | 0.86× (0.85–0.87) | 0.80× (0.79–0.82) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.27–0.29) | 0.97× (0.97–0.97) | 0.77× (0.77–0.78) | 10.95× (10.93–10.97) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.33× (0.32–0.34) | 0.50× (0.41–0.60) | 0.68× (0.65–0.71) | 0.56× (0.54–0.59) | 5.40× (5.16–5.66) | 0.13× (0.12–0.14) | 0.14× (0.13–0.15) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 94 MB | 0.33× (0.32–0.35) | 0.50× (0.44–0.58) | 0.59× (0.55–0.62) | 0.52× (0.49–0.56) | 3.46× (3.27–3.67) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.27× (4.26–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.36× (4.35–4.37) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.76× (1.76–1.76) | 0.84× (0.83–0.84) | 4.36× (4.36–4.37) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.42× (0.41–0.44) | 1.69× (1.68–1.69) | 0.65× (0.65–0.66) | 7.95× (7.92–7.97) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.69× (0.68–0.70) | 0.65× (0.59–0.73) | 1.63× (1.60–1.66) | 0.82× (0.80–0.83) | 5.92× (5.82–6.02) | 0.48× (0.47–0.49) | 0.51× (0.50–0.52) | 0.55× (0.54–0.56) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.62–0.66) | 0.72× (0.67–0.78) | 1.14× (1.11–1.18) | 0.70× (0.68–0.72) | 3.40× (3.30–3.51) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.16× (1.16–1.17) | 0.73× (0.73–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.50× (0.48–0.53) | 0.39× (0.36–0.43) | 0.90× (0.86–0.94) | 0.68× (0.64–0.72) | 3.09× (2.97–3.21) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.80–0.83) | 2.55× (2.50–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.48× (0.46–0.49) | 0.36× (0.34–0.39) | 0.83× (0.80–0.86) | 0.68× (0.66–0.71) | 2.84× (2.74–2.94) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.27–0.30) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.48× (0.45–0.51) | 0.82× (0.80–0.83) | 0.83× (0.79–0.86) | 2.34× (2.29–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.06× (1.06–1.07) | 0.76× (0.76–0.77) | 5.33× (5.31–5.34) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.25–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.77) | 5.27× (5.25–5.29) | 0.34× (0.34–0.35) | 0.36× (0.35–0.37) | 0.90× (0.89–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.28× (5.26–5.29) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 1 | 20 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.66× (0.66–0.67) | 6.34× (6.31–6.36) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.46) | 0.38× (0.35–0.42) | 1.02× (1.00–1.04) | 0.67× (0.64–0.70) | 3.31× (3.26–3.35) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.62 MB → 13.3 MB, 5.1×) | 4 | 55 MB | 0.51× (0.50–0.53) | 0.48× (0.44–0.52) | 0.93× (0.91–0.96) | 0.77× (0.73–0.81) | 2.38× (2.33–2.42) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 539 | 1.86 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 946 | 2.02 | 1.09× (1.08–1.09) | 85% | 1.06× (1.05–1.07) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,146 | 3.17 | 1.69× (1.66–1.71) | 53% | 1.49× (1.48–1.50) | 1.66× (1.63–1.68) |

**Best:** mibz2 at 4 threads (1,146 MB/s); lbzip2 at 4 threads (177 MB/s); pbz2 at 4 threads (325 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.20, lbzip2 1t 0.04, lbzip2 2t 0.28, lbzip2 4t 0.31, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.49 | 0.02 | 0.08 | 0.183 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3900 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 10.7% |
| MTF + RLE2 | 1.63 | 15.1% |
| T-vector build | 1.55 | 14.4% |
| walk (the one each build uses) | 6.05 | 56.1% |
| of which: segment assembly | 0.31 | 2.9% |
| RLE1 | 0.25 | 2.3% |
| CRC | 0.14 | 1.3% |
| **total** | 10.78 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.373 | 0.932 | (in its total) | 2.50× (2.50–2.50) |  |
| walk | 1.476 | 2.793 | (in its total) | 1.89× (1.89–1.90) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.080 | 0.073 | (in its total) | 0.91× (0.91–0.92) |  |
| inverse BWT, total | 1.929 | 3.799 | 2.002 | 1.97× (1.97–1.97) | 1.04× (1.04–1.04) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m37s ± 6m29s |
| 4 threads | 26m44s ± 3m46s |
| 4 threads, the sample's overlap gap as a fixed cost | 17m52s ± 2m32s |
| the scanner alone (its serial part) | 10m20s ± 19.7s |
| the workers alone, over 4 | 17m52s ± 2m32s |

At 4 threads the workers bound it. Per byte: the scanner 6.0 ns a byte of bz2, the workers 2.34 ns a byte of output at 4 threads (CPU, summed over them), 1.52 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,615,664 | 13,288,779 | \`0b9f69f4e27fcf7e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3899.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261116.88.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
