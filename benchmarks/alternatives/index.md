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

The latest record: [`5257865`](<https://github.com/lmmx/mibz2/commit/5257865fd5c932dfa464c68847298e0a71e40e7e>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44889264249>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.532 s | 394 | 9.0 | 3.04× (2.94–3.15) | 6.68× (6.47–6.89) | 7.73× (7.53–7.95) | 7.90× (7.62–8.18) | 7.07× (6.80–7.34) | 7.97× (7.71–8.24) | 7.31× (7.11–7.51) | 2.22× (2.16–2.28) | 2.28× (2.25–2.30) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.311 s | 675 | 9.8 | 3.02× (2.87–3.16) | 6.23× (5.88–6.61) | 7.99× (7.48–8.52) | 7.28× (6.91–7.67) | 6.66× (6.38–6.95) | 13.64× (12.90–14.43) | 12.50× (11.96–13.07) | 3.80× (3.65–3.97) | 3.89× (3.78–4.02) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.267 s | 785 | 16.3 | 3.33× (3.27–3.38) | 4.69× (4.39–5.02) | 6.26× (5.98–6.55) | 5.56× (5.26–5.87) | 5.57× (5.35–5.80) | 15.95× (15.37–16.54) | 14.61× (14.18–15.05) | 4.45× (4.32–4.57) | 4.55× (4.47–4.63) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,015 | 3.8 | 0.67× (0.65–0.69) | 3.16× (2.99–3.33) | 5.62× (5.37–5.87) | 3.64× (3.49–3.79) | 6.52× (6.22–6.84) | 2.78× (2.67–2.90) | 2.70× (2.58–2.83) | 1.18× (1.12–1.24) | 1.05× (1.00–1.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 978 | 4.0 | 0.62× (0.60–0.64) | 3.01× (2.93–3.09) | 6.93× (6.74–7.12) | 3.51× (3.42–3.61) | 6.35× (6.20–6.51) | 2.69× (2.61–2.77) | 2.61× (2.54–2.69) | 1.14× (1.10–1.18) | 1.02× (0.99–1.04) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.020 s | 980 | 3.9 | 0.63× (0.61–0.64) | 3.04× (2.98–3.09) | 7.01× (6.86–7.15) | 3.51× (3.45–3.57) | 6.46× (6.38–6.55) | 2.70× (2.65–2.75) | 2.63× (2.58–2.67) | 1.15× (1.12–1.18) | 1.02× (1.00–1.04) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 548 | 6.8 | 3.12× (3.02–3.22) | 3.41× (3.33–3.50) | 4.54× (4.44–4.64) | 4.24× (4.14–4.35) | 4.90× (4.77–5.03) | 3.90× (3.80–4.00) | 3.03× (2.96–3.10) | 2.20× (2.14–2.25) | 2.31× (2.24–2.38) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 841 | 8.2 | 2.58× (2.45–2.73) | 2.96× (2.74–3.20) | 4.70× (4.46–4.96) | 3.70× (3.52–3.90) | 5.43× (5.17–5.71) | 5.93× (5.66–6.22) | 4.61× (4.41–4.82) | 3.34× (3.21–3.48) | 3.51× (3.33–3.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 773 | 15.8 | 2.38× (2.30–2.45) | 1.98× (1.85–2.11) | 3.28× (3.10–3.48) | 2.40× (2.23–2.59) | 4.38× (4.15–4.62) | 5.43× (5.18–5.69) | 4.22× (4.02–4.43) | 3.06× (2.95–3.17) | 3.21× (3.03–3.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.153 s | 52 | 68.2 | 1.33× (1.30–1.36) | 2.45× (2.39–2.51) | 2.30× (2.22–2.39) | 2.29× (2.21–2.38) | 2.68× (2.60–2.77) | 3.03× (2.95–3.12) | 2.96× (2.87–3.06) | 1.23× (1.20–1.25) | 1.24× (1.22–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.092 s | 87 | 75.5 | 1.23× (1.13–1.33) | 2.36× (2.16–2.57) | 2.26× (2.18–2.34) | 2.82× (2.66–3.00) | 2.71× (2.54–2.88) | 4.90× (4.63–5.19) | 4.79× (4.53–5.06) | 1.98× (1.84–2.13) | 2.01× (1.89–2.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.078 s | 103 | 104.2 | 1.44× (1.40–1.48) | 2.17× (2.06–2.29) | 2.10× (2.04–2.17) | 2.61× (2.55–2.68) | 2.80× (2.74–2.86) | 5.98× (5.79–6.19) | 5.84× (5.62–6.07) | 2.42× (2.31–2.53) | 2.45× (2.36–2.55) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.172 s | 56 | 63.9 | 1.38× (1.33–1.43) | 2.71× (2.62–2.81) | 2.43× (2.31–2.54) | 2.45× (2.33–2.57) | 2.82× (2.70–2.93) | 3.61× (3.49–3.74) | 3.71× (3.58–3.85) | 1.39× (1.33–1.46) | 1.44× (1.40–1.49) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.101 s | 95 | 70.4 | 1.33× (1.30–1.36) | 2.61× (2.53–2.69) | 2.39× (2.30–2.49) | 3.37× (3.30–3.44) | 2.95× (2.81–3.09) | 6.25× (6.07–6.43) | 6.42× (6.24–6.62) | 2.41× (2.32–2.50) | 2.50× (2.41–2.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.090 s | 107 | 104.0 | 1.56× (1.53–1.58) | 2.26× (2.19–2.32) | 2.18× (2.10–2.26) | 2.93× (2.86–2.99) | 2.75× (2.69–2.80) | 7.07× (6.97–7.17) | 7.27× (7.14–7.40) | 2.73× (2.68–2.78) | 2.83× (2.76–2.89) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 778 | 5.2 | 0.72× (0.67–0.77) | 3.19× (3.10–3.27) | 5.15× (4.82–5.51) | 3.24× (3.16–3.32) | 7.53× (7.32–7.75) | 2.43× (2.36–2.51) | 2.43× (2.36–2.51) | 1.07× (1.03–1.11) | 1.61× (1.57–1.65) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 709 | 5.8 | 0.63× (0.60–0.67) | 2.92× (2.86–2.97) | 5.71× (5.61–5.82) | 3.03× (2.95–3.11) | 6.98× (6.77–7.19) | 2.22× (2.16–2.27) | 2.22× (2.17–2.26) | 0.98× (0.95–1.00) | 1.47× (1.44–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 692 | 5.7 | 0.59× (0.56–0.62) | 2.86× (2.79–2.93) | 5.67× (5.45–5.89) | 2.94× (2.85–3.02) | 6.79× (6.61–6.97) | 2.17× (2.11–2.23) | 2.17× (2.12–2.22) | 0.95× (0.91–1.00) | 1.44× (1.40–1.47) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.079 s | 169 | 21.4 | 1.81× (1.77–1.85) | 3.69× (3.57–3.81) | 4.36× (4.24–4.47) | 4.22× (4.11–4.34) | 4.38× (4.30–4.47) | 4.77× (4.66–4.89) | 4.52× (4.37–4.69) | 1.41× (1.38–1.44) | 1.42× (1.40–1.45) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.051 s | 264 | 24.7 | 1.62× (1.54–1.70) | 3.35× (3.05–3.68) | 4.43× (4.07–4.82) | 4.50× (4.20–4.83) | 4.47× (4.22–4.74) | 7.46× (7.12–7.82) | 7.07× (6.70–7.46) | 2.20× (2.10–2.30) | 2.22× (2.12–2.33) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.045 s | 294 | 39.0 | 1.81× (1.75–1.86) | 2.70× (2.55–2.86) | 3.53× (3.31–3.75) | 3.64× (3.43–3.86) | 4.19× (4.00–4.40) | 8.38× (8.13–8.64) | 7.94× (7.74–8.14) | 2.47× (2.41–2.53) | 2.49× (2.43–2.56) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.03× (2.93–3.14) | 6.65× (6.44–6.86) | 7.71× (7.50–7.92) | 7.87× (7.60–8.16) | 7.05× (6.78–7.32) | 7.95× (7.69–8.21) | 7.28× (7.09–7.48) | 2.22× (2.16–2.28) | 2.26× (2.23–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.94× (2.86–3.02) | 6.42× (6.21–6.64) | 7.61× (7.30–7.94) | 7.72× (7.49–7.96) | 6.76× (6.58–6.94) | 7.29× (7.06–7.53) | 6.68× (6.50–6.87) | 2.04× (1.99–2.09) | 2.08× (2.05–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.25× (3.15–3.34) | 5.01× (4.73–5.31) | 6.05× (5.78–6.34) | 5.90× (5.64–6.18) | 5.39× (5.20–5.58) | 4.35× (4.23–4.48) | 3.99× (3.88–4.10) | 1.22× (1.18–1.25) | 1.24× (1.22–1.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.69× (0.67–0.71) | 2.85× (2.73–2.97) | 5.08× (4.87–5.30) | 3.43× (3.30–3.56) | 6.08× (5.82–6.34) | 2.64× (2.54–2.73) | 2.57× (2.47–2.68) | 1.17× (1.12–1.23) | 0.98× (0.93–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.66× (0.64–0.68) | 2.72× (2.63–2.80) | 6.20× (6.03–6.37) | 3.28× (3.19–3.37) | 5.85× (5.69–6.01) | 2.51× (2.43–2.59) | 2.45× (2.38–2.53) | 1.11× (1.07–1.16) | 0.94× (0.92–0.95) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.65× (0.64–0.67) | 3.15× (3.09–3.21) | 6.34× (6.25–6.44) | 3.33× (3.27–3.39) | 6.02× (5.94–6.10) | 2.56× (2.52–2.61) | 2.50× (2.46–2.54) | 1.14× (1.11–1.17) | 0.95× (0.92–0.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 3.02× (2.93–3.12) | 3.11× (3.03–3.20) | 4.29× (4.20–4.39) | 4.10× (4.00–4.20) | 4.73× (4.60–4.85) | 3.77× (3.67–3.87) | 2.94× (2.88–3.00) | 2.14× (2.09–2.20) | 2.13× (2.09–2.18) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.65× (2.55–2.76) | 2.76× (2.67–2.86) | 3.86× (3.74–3.98) | 3.49× (3.37–3.60) | 4.03× (3.87–4.18) | 3.11× (3.00–3.23) | 2.43× (2.34–2.52) | 1.77× (1.71–1.83) | 1.76× (1.70–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.60× (2.53–2.67) | 1.85× (1.77–1.92) | 2.67× (2.60–2.74) | 2.21× (2.12–2.29) | 2.46× (2.37–2.54) | 1.60× (1.55–1.65) | 1.25× (1.21–1.29) | 0.91× (0.89–0.93) | 0.91× (0.88–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.30–1.35) | 2.40× (2.35–2.46) | 2.25× (2.16–2.34) | 2.28× (2.20–2.36) | 2.66× (2.58–2.75) | 3.01× (2.93–3.10) | 2.94× (2.85–3.03) | 1.22× (1.20–1.25) | 1.21× (1.18–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.23–1.29) | 2.32× (2.27–2.37) | 2.15× (2.07–2.24) | 2.75× (2.70–2.79) | 2.46× (2.41–2.52) | 2.73× (2.66–2.81) | 2.67× (2.60–2.74) | 1.11× (1.07–1.15) | 1.10× (1.08–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.54–1.61) | 2.32× (2.17–2.47) | 1.95× (1.88–2.03) | 2.58× (2.50–2.67) | 2.42× (2.37–2.47) | 1.97× (1.90–2.04) | 1.93× (1.85–2.01) | 0.80× (0.77–0.84) | 0.79× (0.76–0.82) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.37× (1.32–1.42) | 2.67× (2.58–2.76) | 2.39× (2.28–2.50) | 2.43× (2.32–2.55) | 2.80× (2.69–2.92) | 3.59× (3.47–3.71) | 3.69× (3.56–3.83) | 1.39× (1.33–1.46) | 1.41× (1.36–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.31–1.35) | 2.62× (2.57–2.67) | 2.25× (2.20–2.30) | 3.13× (3.07–3.20) | 2.66× (2.56–2.76) | 3.31× (3.24–3.37) | 3.40× (3.34–3.47) | 1.28× (1.25–1.32) | 1.30× (1.27–1.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.73× (1.68–1.77) | 2.44× (2.37–2.52) | 1.94× (1.90–1.98) | 2.70× (2.62–2.79) | 2.44× (2.39–2.49) | 2.24× (2.18–2.29) | 2.30× (2.24–2.36) | 0.87× (0.84–0.89) | 0.88× (0.85–0.91) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.75× (0.70–0.80) | 2.49× (2.42–2.56) | 4.28× (4.14–4.42) | 2.97× (2.89–3.05) | 6.74× (6.53–6.96) | 2.25× (2.18–2.33) | 2.25× (2.18–2.33) | 1.06× (1.02–1.09) | 0.90× (0.85–0.95) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.68× (0.66–0.71) | 2.29× (2.24–2.33) | 4.85× (4.77–4.94) | 2.81× (2.74–2.89) | 6.21× (6.05–6.38) | 2.04× (1.98–2.09) | 2.04× (1.99–2.08) | 0.95× (0.93–0.98) | 0.81× (0.77–0.86) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.63× (0.60–0.66) | 2.62× (2.55–2.70) | 4.88× (4.73–5.02) | 2.80× (2.73–2.88) | 6.22× (6.07–6.37) | 2.05× (1.99–2.10) | 2.05× (2.00–2.09) | 0.96× (0.92–1.00) | 0.82× (0.77–0.87) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.79× (1.75–1.83) | 3.56× (3.46–3.67) | 4.22× (4.10–4.34) | 4.15× (4.04–4.26) | 4.31× (4.22–4.39) | 4.69× (4.58–4.80) | 4.44× (4.29–4.60) | 1.40× (1.37–1.43) | 1.37× (1.34–1.40) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.67× (1.64–1.70) | 3.34× (3.11–3.58) | 4.06× (3.76–4.39) | 4.26× (4.18–4.34) | 3.98× (3.78–4.18) | 4.07× (3.96–4.18) | 3.85× (3.73–3.98) | 1.21× (1.18–1.24) | 1.19× (1.17–1.21) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.95× (1.86–2.04) | 2.78× (2.59–3.00) | 3.29× (3.04–3.56) | 3.49× (3.36–3.62) | 3.34× (3.10–3.60) | 2.60× (2.51–2.68) | 2.46× (2.36–2.56) | 0.77× (0.75–0.80) | 0.76× (0.74–0.77) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.26× (0.26–0.26) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.96× (10.93–10.99) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.48× (0.40–0.59) | 0.68× (0.66–0.70) | 0.58× (0.57–0.60) | 5.46× (5.32–5.60) | 0.13× (0.13–0.13) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.35× (0.33–0.37) | 0.52× (0.46–0.59) | 0.61× (0.57–0.65) | 0.55× (0.52–0.58) | 3.59× (3.41–3.77) | 0.08× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.28× (4.26–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.44) | 1.68× (1.67–1.69) | 0.66× (0.65–0.66) | 7.94× (7.91–7.97) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.62× (0.54–0.70) | 1.62× (1.61–1.63) | 0.81× (0.80–0.82) | 5.89× (5.84–5.95) | 0.48× (0.47–0.48) | 0.50× (0.50–0.51) | 0.55× (0.54–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.65× (0.64–0.65) | 0.70× (0.66–0.74) | 1.15× (1.14–1.17) | 0.71× (0.70–0.73) | 3.46× (3.42–3.49) | 0.27× (0.27–0.27) | 0.29× (0.28–0.29) | 0.31× (0.31–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.14× (5.12–5.15) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.52) | 0.37× (0.34–0.41) | 0.90× (0.88–0.92) | 0.71× (0.69–0.74) | 3.10× (3.02–3.18) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.46× (0.44–0.47) | 0.86× (0.84–0.89) | 0.80× (0.78–0.83) | 2.51× (2.44–2.59) | 0.15× (0.14–0.15) | 0.16× (0.15–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.83× (0.82–0.83) | 5.18× (5.16–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.48–0.51) | 0.37× (0.34–0.40) | 0.85× (0.83–0.87) | 0.74× (0.72–0.76) | 2.93× (2.85–3.01) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.60× (0.59–0.61) | 0.47× (0.44–0.51) | 0.81× (0.80–0.83) | 0.82× (0.79–0.86) | 2.35× (2.31–2.39) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.35× (5.33–5.36) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.58× (1.58–1.59) | 0.76× (0.76–0.76) | 5.26× (5.25–5.27) | 0.34× (0.34–0.34) | 0.37× (0.37–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.76× (0.76–0.77) | 5.28× (5.27–5.29) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.39–1.40) | 0.66× (0.66–0.67) | 6.36× (6.34–6.37) | 0.39× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.44× (0.43–0.46) | 0.38× (0.34–0.42) | 1.03× (1.00–1.06) | 0.66× (0.64–0.68) | 3.27× (3.20–3.35) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.15–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.48× (0.43–0.54) | 0.90× (0.88–0.93) | 0.74× (0.72–0.77) | 2.31× (2.25–2.36) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 394 | 2.54 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 675 | 2.78 | 1.09× (1.08–1.10) | 86% | 1.05× (1.03–1.08) | 1.06× (1.03–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 785 | 4.62 | 1.83× (1.80–1.85) | 50% | 1.38× (1.33–1.43) | 1.95× (1.90–2.01) |

**Best:** mibz2 at 4 threads (785 MB/s); lbzip2 at 4 threads (174 MB/s); pbz2 at 4 threads (238 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.17, lbzip2 1t 0.05, lbzip2 2t 0.27, lbzip2 4t 0.32, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.35. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.75 | 0.03 | 0.08 | 0.267 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3536 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.27 | 10.5% |
| MTF + RLE2 | 1.77 | 14.6% |
| T-vector build | 1.77 | 14.6% |
| walk (the one each build uses) | 6.89 | 56.7% |
| of which: segment assembly | 0.33 | 2.7% |
| RLE1 | 0.28 | 2.3% |
| CRC | 0.16 | 1.3% |
| **total** | 12.16 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.489 | 1.272 | (in its total) | 2.60× (2.60–2.60) |  |
| walk | 1.848 | 2.964 | (in its total) | 1.61× (1.60–1.61) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.092 | 0.092 | (in its total) | 1.01× (0.97–1.05) |  |
| inverse BWT, total | 2.429 | 4.327 | 2.552 | 1.78× (1.78–1.79) | 1.05× (1.05–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 8m50s |
| 4 threads | 38m48s ± 5m28s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m47s ± 3m57s |
| the scanner alone (its serial part) | 14m59s ± 30.2s |
| the workers alone, over 4 | 27m47s ± 3m57s |

At 4 threads the workers bound it. Per byte: the scanner 8.7 ns a byte of bz2, the workers 3.64 ns a byte of output at 4 threads (CPU, summed over them), 2.05 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,633,100 | 13,390,258 | \`b7ebdafdab81f6ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 3535.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
