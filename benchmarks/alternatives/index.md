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

The latest record: [`a4f9771`](<https://github.com/lmmx/mibz2/commit/a4f97714cc14366984015869d9d094d89488ac57>) (2026-11-08).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42521128791>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.358 s | 585 | 7.8 | 2.35× (2.31–2.40) | 5.95× (5.86–6.04) | 6.22× (6.09–6.36) | 7.08× (6.96–7.20) | 6.32× (6.20–6.45) | 6.87× (6.47–7.29) | 6.15× (5.99–6.32) | 1.86× (1.83–1.90) | 2.10× (2.07–2.13) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.208 s | 1,008 | 8.8 | 2.30× (2.16–2.45) | 5.19× (4.88–5.52) | 6.35× (5.90–6.83) | 6.26× (5.81–6.75) | 5.56× (5.20–5.94) | 11.28× (10.21–12.46) | 10.10× (9.34–10.93) | 3.06× (2.84–3.30) | 3.45× (3.21–3.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.195 s | 1,074 | 15.4 | 2.26× (2.16–2.37) | 3.52× (3.37–3.67) | 4.59× (4.38–4.82) | 4.22× (4.05–4.41) | 4.12× (4.00–4.24) | 12.60× (11.66–13.62) | 11.29× (10.76–11.84) | 3.42× (3.27–3.58) | 3.85× (3.71–4.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,384 | 3.7 | 0.55× (0.54–0.56) | 3.55× (3.47–3.63) | 4.38× (4.27–4.49) | 3.64× (3.56–3.71) | 6.42× (6.29–6.56) | 2.72× (2.66–2.78) | 2.72× (2.66–2.79) | 0.99× (0.97–1.01) | 2.13× (2.08–2.18) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,360 | 3.8 | 0.51× (0.49–0.52) | 2.80× (2.71–2.88) | 5.71× (5.55–5.88) | 3.59× (3.49–3.70) | 6.18× (5.98–6.39) | 2.67× (2.58–2.75) | 2.67× (2.57–2.77) | 0.97× (0.95–1.00) | 2.09× (2.03–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,320 | 3.8 | 0.50× (0.48–0.51) | 2.74× (2.67–2.80) | 5.87× (5.32–6.47) | 3.50× (3.39–3.61) | 6.16× (5.99–6.32) | 2.60× (2.54–2.67) | 2.61× (2.54–2.68) | 0.95× (0.93–0.97) | 2.04× (1.99–2.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 982 | 5.0 | 2.62× (2.54–2.70) | 3.39× (3.03–3.78) | 4.02× (3.82–4.23) | 4.25× (4.18–4.31) | 4.94× (4.84–5.04) | 2.58× (2.52–2.65) | 2.38× (2.34–2.41) | 1.99× (1.95–2.02) | 3.06× (2.99–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.015 s | 1,356 | 6.7 | 1.98× (1.73–2.27) | 2.76× (2.44–3.13) | 3.87× (3.33–4.49) | 3.63× (3.46–3.81) | 5.20× (4.56–5.93) | 3.46× (3.04–3.95) | 3.19× (2.81–3.61) | 2.66× (2.34–3.03) | 4.10× (3.62–4.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,143 | 13.2 | 1.70× (1.51–1.92) | 2.08× (1.72–2.52) | 2.58× (2.49–2.68) | 2.00× (1.93–2.08) | 4.07× (3.93–4.21) | 2.98× (2.86–3.12) | 2.74× (2.68–2.81) | 2.30× (2.22–2.38) | 3.53× (3.42–3.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.080 s | 100 | 46.4 | 1.38× (1.36–1.40) | 3.16× (3.06–3.27) | 2.86× (2.79–2.93) | 2.81× (2.72–2.90) | 3.16× (3.06–3.26) | 3.69× (3.58–3.80) | 3.55× (3.45–3.65) | 1.35× (1.32–1.39) | 1.53× (1.48–1.59) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 153 | 53.9 | 1.22× (1.17–1.27) | 2.89× (2.72–3.06) | 2.64× (2.52–2.76) | 3.33× (3.17–3.51) | 3.06× (2.94–3.17) | 5.60× (5.35–5.87) | 5.39× (5.20–5.59) | 2.06× (1.97–2.15) | 2.33× (2.25–2.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 184 | 71.9 | 1.24× (1.17–1.31) | 2.37× (2.26–2.49) | 2.35× (2.23–2.47) | 2.77× (2.61–2.94) | 3.11× (3.01–3.21) | 6.64× (6.22–7.09) | 6.39× (6.03–6.78) | 2.44× (2.29–2.60) | 2.76× (2.60–2.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.085 s | 114 | 40.7 | 1.50× (1.45–1.55) | 3.80× (3.69–3.92) | 3.08× (3.03–3.14) | 3.11× (3.06–3.15) | 3.51× (3.46–3.56) | 4.66× (4.62–4.69) | 4.73× (4.61–4.86) | 1.62× (1.59–1.64) | 1.84× (1.80–1.87) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.056 s | 171 | 49.8 | 1.29× (1.25–1.34) | 3.38× (3.23–3.54) | 2.84× (2.74–2.95) | 3.77× (3.53–4.04) | 3.30× (3.20–3.41) | 7.12× (6.88–7.38) | 7.24× (6.95–7.55) | 2.47× (2.38–2.57) | 2.81× (2.72–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.046 s | 207 | 66.7 | 1.39× (1.35–1.44) | 2.65× (2.55–2.75) | 2.60× (2.51–2.69) | 3.40× (3.27–3.54) | 3.20× (3.09–3.32) | 8.61× (8.35–8.87) | 8.75× (8.38–9.14) | 2.99× (2.87–3.11) | 3.39× (3.28–3.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,020 | 5.2 | 0.60× (0.59–0.61) | 3.14× (3.05–3.23) | 4.36× (4.23–4.49) | 3.11× (3.03–3.19) | 6.89× (6.74–7.04) | 2.27× (2.23–2.32) | 2.29× (2.25–2.34) | 0.96× (0.94–0.99) | 2.10× (2.04–2.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,010 | 5.3 | 0.50× (0.49–0.51) | 3.06× (2.95–3.18) | 5.17× (4.88–5.49) | 2.99× (2.92–3.06) | 6.81× (6.63–6.99) | 2.21× (2.17–2.26) | 2.23× (2.17–2.29) | 0.93× (0.91–0.96) | 2.04× (1.97–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 952 | 5.4 | 0.49× (0.48–0.50) | 2.96× (2.88–3.05) | 5.10× (4.94–5.27) | 2.92× (2.85–2.99) | 6.56× (6.37–6.76) | 2.13× (2.10–2.17) | 2.15× (2.11–2.20) | 0.90× (0.89–0.92) | 1.97× (1.92–2.03) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 0.054 s | 231 | 20.2 | 1.29× (1.26–1.31) | 3.23× (3.16–3.29) | 3.47× (3.41–3.53) | 3.54× (3.47–3.62) | 3.85× (3.76–3.93) | 4.09× (4.02–4.17) | 3.59× (3.53–3.64) | 1.13× (1.12–1.15) | 1.31× (1.29–1.34) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 0.036 s | 344 | 24.6 | 1.10× (1.00–1.21) | 2.72× (2.55–2.90) | 3.25× (2.96–3.57) | 3.38× (3.14–3.65) | 3.57× (3.33–3.82) | 5.98× (5.54–6.46) | 5.25× (4.89–5.63) | 1.66× (1.53–1.79) | 1.92× (1.78–2.07) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 0.030 s | 412 | 36.5 | 1.17× (1.12–1.23) | 2.22× (2.07–2.39) | 2.90× (2.73–3.07) | 3.07× (2.84–3.33) | 3.56× (3.31–3.83) | 7.33× (7.06–7.61) | 6.43× (6.20–6.66) | 2.03× (1.94–2.12) | 2.35× (2.25–2.45) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.35× (2.30–2.39) | 5.91× (5.82–6.00) | 6.19× (6.05–6.32) | 7.05× (6.94–7.17) | 6.30× (6.17–6.43) | 6.84× (6.44–7.26) | 6.13× (5.97–6.29) | 1.86× (1.82–1.90) | 2.08× (2.06–2.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.16× (2.12–2.20) | 5.42× (5.27–5.58) | 5.80× (5.62–5.99) | 6.57× (6.39–6.77) | 5.58× (5.42–5.74) | 6.04× (5.63–6.48) | 5.41× (5.20–5.64) | 1.64× (1.58–1.70) | 1.84× (1.78–1.90) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.08× (1.98–2.19) | 3.71× (3.57–3.85) | 4.12× (3.97–4.27) | 4.49× (4.33–4.67) | 3.80× (3.69–3.92) | 3.43× (3.17–3.70) | 3.07× (2.92–3.23) | 0.93× (0.89–0.98) | 1.04× (1.00–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.58–0.61) | 2.98× (2.88–3.09) | 3.82× (3.72–3.93) | 3.36× (3.28–3.43) | 5.85× (5.73–5.97) | 2.54× (2.49–2.58) | 2.54× (2.48–2.59) | 0.99× (0.97–1.01) | 1.78× (1.75–1.81) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.58) | 2.73× (2.66–2.81) | 4.87× (4.75–4.98) | 3.31× (3.22–3.41) | 5.59× (5.41–5.77) | 2.46× (2.39–2.53) | 2.46× (2.38–2.55) | 0.96× (0.94–0.98) | 1.73× (1.68–1.78) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.55) | 2.95× (2.88–3.03) | 5.07× (4.60–5.57) | 3.31× (3.21–3.40) | 5.68× (5.54–5.83) | 2.47× (2.41–2.52) | 2.47× (2.41–2.53) | 0.96× (0.94–0.99) | 1.73× (1.68–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.49× (2.42–2.56) | 3.07× (2.77–3.41) | 3.50× (3.42–3.59) | 3.99× (3.93–4.06) | 4.63× (4.54–4.73) | 2.46× (2.40–2.52) | 2.27× (2.23–2.30) | 1.91× (1.88–1.94) | 2.54× (2.48–2.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.03× (1.91–2.15) | 2.41× (2.27–2.56) | 2.86× (2.71–3.02) | 3.38× (3.08–3.70) | 3.65× (3.43–3.87) | 1.83× (1.73–1.94) | 1.69× (1.61–1.78) | 1.42× (1.35–1.50) | 1.90× (1.80–2.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.90× (1.67–2.15) | 1.92× (1.49–2.49) | 1.96× (1.86–2.07) | 1.96× (1.90–2.03) | 2.14× (2.05–2.24) | 0.95× (0.90–1.00) | 0.87× (0.83–0.92) | 0.73× (0.69–0.78) | 0.98× (0.93–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.35–1.39) | 3.07× (2.98–3.16) | 2.76× (2.69–2.83) | 2.78× (2.69–2.87) | 3.12× (3.01–3.23) | 3.64× (3.53–3.75) | 3.50× (3.40–3.59) | 1.35× (1.31–1.38) | 1.46× (1.40–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.24× (1.21–1.27) | 2.86× (2.70–3.02) | 2.40× (2.33–2.46) | 3.13× (3.08–3.19) | 2.71× (2.66–2.77) | 3.11× (3.02–3.21) | 2.99× (2.93–3.06) | 1.15× (1.12–1.19) | 1.25× (1.22–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.33–1.47) | 2.53× (2.38–2.70) | 2.09× (1.95–2.25) | 2.77× (2.64–2.91) | 2.69× (2.62–2.77) | 2.30× (2.16–2.44) | 2.21× (2.08–2.34) | 0.85× (0.80–0.90) | 0.92× (0.87–0.98) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.44–1.54) | 3.69× (3.57–3.82) | 3.00× (2.95–3.05) | 3.07× (3.02–3.11) | 3.46× (3.42–3.51) | 4.58× (4.55–4.62) | 4.66× (4.54–4.79) | 1.60× (1.58–1.63) | 1.75× (1.73–1.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.29–1.35) | 3.25× (3.14–3.36) | 2.60× (2.47–2.74) | 3.54× (3.39–3.70) | 2.97× (2.89–3.05) | 3.84× (3.71–3.97) | 3.90× (3.74–4.07) | 1.34× (1.30–1.39) | 1.47× (1.42–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.51–1.58) | 2.90× (2.79–3.00) | 2.24× (2.18–2.30) | 3.12× (3.03–3.21) | 2.84× (2.75–2.93) | 2.80× (2.71–2.89) | 2.85× (2.71–2.99) | 0.98× (0.94–1.02) | 1.07× (1.03–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.47× (2.41–2.53) | 3.30× (3.23–3.37) | 2.81× (2.75–2.88) | 6.06× (5.94–6.18) | 2.09× (2.06–2.13) | 2.10× (2.07–2.14) | 0.96× (0.94–0.98) | 1.45× (1.43–1.47) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.57–0.60) | 2.29× (2.25–2.33) | 3.98× (3.88–4.09) | 2.71× (2.66–2.77) | 5.85× (5.73–5.98) | 1.99× (1.96–2.02) | 2.00× (1.95–2.05) | 0.92× (0.90–0.93) | 1.38× (1.35–1.41) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.55–0.57) | 2.41× (2.35–2.47) | 3.97× (3.92–4.02) | 2.76× (2.69–2.82) | 5.90× (5.72–6.09) | 2.01× (1.97–2.04) | 2.02× (1.98–2.06) | 0.92× (0.91–0.94) | 1.39× (1.35–1.43) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 1.28× (1.25–1.30) | 3.04× (2.97–3.12) | 3.31× (3.25–3.38) | 3.47× (3.39–3.54) | 3.76× (3.68–3.85) | 4.00× (3.93–4.08) | 3.51× (3.46–3.56) | 1.13× (1.11–1.15) | 1.22× (1.19–1.24) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 1.15× (1.08–1.23) | 2.64× (2.51–2.77) | 2.86× (2.73–3.00) | 3.22× (3.09–3.36) | 3.12× (2.99–3.24) | 3.24× (3.11–3.38) | 2.84× (2.74–2.95) | 0.91× (0.87–0.96) | 0.98× (0.94–1.03) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 1.25× (1.21–1.29) | 2.18× (2.07–2.29) | 2.45× (2.33–2.58) | 2.63× (2.54–2.73) | 2.67× (2.54–2.81) | 2.22× (2.14–2.31) | 1.95× (1.88–2.02) | 0.63× (0.60–0.66) | 0.68× (0.65–0.70) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.37× (0.36–0.39) | 1.08× (1.08–1.08) | 0.87× (0.86–0.88) | 12.19× (12.17–12.21) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.61× (0.53–0.70) | 0.78× (0.77–0.80) | 0.65× (0.64–0.67) | 6.28× (6.17–6.39) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.38–0.39) | 0.66× (0.51–0.85) | 0.71× (0.70–0.72) | 0.61× (0.60–0.61) | 4.07× (4.04–4.10) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.26× (0.26–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.83–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.47× (0.45–0.48) | 1.73× (1.72–1.74) | 0.67× (0.66–0.67) | 8.15× (8.12–8.18) | 0.64× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.70) | 0.68× (0.64–0.71) | 1.62× (1.60–1.65) | 0.80× (0.79–0.82) | 5.89× (5.81–5.96) | 0.46× (0.46–0.47) | 0.49× (0.49–0.50) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.62× (0.60–0.65) | 0.70× (0.66–0.74) | 1.12× (1.07–1.17) | 0.73× (0.70–0.76) | 3.27× (3.13–3.42) | 0.25× (0.24–0.26) | 0.26× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.74× (0.74–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.46–0.51) | 0.41× (0.39–0.44) | 0.90× (0.86–0.94) | 0.70× (0.66–0.74) | 3.12× (2.98–3.26) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.29–0.32) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.44–0.47) | 0.87× (0.84–0.90) | 0.80× (0.77–0.84) | 2.54× (2.47–2.61) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.83× (0.83–0.83) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.44–0.48) | 0.38× (0.37–0.40) | 0.83× (0.80–0.86) | 0.70× (0.68–0.72) | 2.85× (2.76–2.96) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.48× (0.46–0.51) | 0.81× (0.79–0.83) | 0.82× (0.79–0.84) | 2.33× (2.29–2.37) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.36× (0.36–0.37) | 0.22× (0.21–0.24) | 1.09× (1.09–1.09) | 0.77× (0.77–0.78) | 5.45× (5.43–5.46) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.39–5.42) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.77) | 5.42× (5.40–5.43) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.41× (0.41–0.41) | 1.49× (1.49–1.50) | 0.67× (0.67–0.67) | 6.76× (6.75–6.77) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 33 MB | 0.49× (0.47–0.50) | 0.42× (0.38–0.47) | 1.17× (1.14–1.20) | 0.75× (0.73–0.77) | 3.69× (3.59–3.79) | 0.22× (0.21–0.23) | 0.23× (0.23–0.24) | 0.36× (0.35–0.37) | 0.18× (0.17–0.18) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 50 MB | 0.54× (0.53–0.56) | 0.49× (0.44–0.55) | 1.01× (0.99–1.04) | 0.79× (0.76–0.83) | 2.58× (2.53–2.64) | 0.15× (0.14–0.15) | 0.15× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 585 | 1.72 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,008 | 1.93 | 1.13× (1.10–1.16) | 82% | 1.04× (1.02–1.06) | 1.04× (1.01–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 1,074 | 3.41 | 2.00× (1.93–2.07) | 46% | 1.25× (1.20–1.30) | 1.77× (1.69–1.86) |

**Best:** mibz2 at 4 threads (1,074 MB/s); lbzip2 at 4 threads (310 MB/s); pbz2 at 4 threads (483 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.84, mibz2 4t 1.14, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.53. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 4%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.56 | 0.01 | 0.09 | 0.195 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4530 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 37.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.4% |
| MTF + RLE2 | 1.50 | 20.9% |
| T-vector build | 1.40 | 19.5% |
| walk (the one each build uses) | 2.84 | 39.6% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.16 | 2.3% |
| CRC | 0.25 | 3.5% |
| **total** | 7.17 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.307 | 0.687 | (in its total) | 2.26× (2.21–2.31) |  |
| walk | 0.630 | 1.549 | (in its total) | 2.51× (2.45–2.58) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.029 | (in its total) | 1.09× (1.02–1.17) |  |
| inverse BWT, total | 0.965 | 2.265 | 1.254 | 2.39× (2.33–2.45) | 1.32× (1.30–1.35) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 52m01s ± 6m05s |
| 4 threads | 28m23s ± 4m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m20s ± 3m06s |
| the scanner alone (its serial part) | 9m01s ± 26.3s |
| the workers alone, over 4 | 20m20s ± 3m06s |

At 4 threads the workers bound it. Per byte: the scanner 5.2 ns a byte of bz2, the workers 2.67 ns a byte of output at 4 threads (CPU, summed over them), 1.41 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,094,880 | 12,489,382 | \`0cafc9c0ea86fcad…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4530.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
