# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-1053/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`cf242fe`](<https://github.com/lmmx/mibz2/commit/cf242fe7a41d1dce1bc4bee59fb974c28c882f77>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45920252963>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.416 s | 504 | 7.2 | 3.38× (3.31–3.45) | 7.53× (7.37–7.69) | 7.93× (7.77–8.10) | 8.97× (8.79–9.16) | 8.07× (7.88–8.25) | 8.58× (8.40–8.76) | 8.25× (8.07–8.43) | 2.64× (2.58–2.70) | 2.41× (2.36–2.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.233 s | 901 | 7.6 | 3.50× (3.32–3.70) | 7.19× (6.82–7.58) | 8.53× (8.08–9.00) | 8.66× (8.22–9.13) | 7.70× (7.29–8.14) | 15.24× (14.45–16.07) | 14.65× (13.87–15.48) | 4.69× (4.44–4.95) | 4.28× (4.05–4.53) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.187 s | 1,119 | 11.9 | 3.70× (3.66–3.74) | 6.54× (6.44–6.63) | 7.34× (7.22–7.46) | 7.39× (7.26–7.53) | 7.27× (7.21–7.33) | 19.19× (18.96–19.42) | 18.45× (18.24–18.66) | 5.91× (5.82–5.99) | 5.39× (5.32–5.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,149 | 3.4 | 0.59× (0.56–0.63) | 3.58× (3.36–3.81) | 4.81× (4.51–5.13) | 3.89× (3.67–4.12) | 6.53× (6.19–6.88) | 2.93× (2.77–3.10) | 2.91× (2.74–3.10) | 1.15× (1.09–1.22) | 1.21× (1.13–1.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,053 | 3.8 | 0.54× (0.50–0.57) | 2.81× (2.65–2.98) | 5.73× (5.38–6.10) | 3.70× (3.49–3.91) | 6.08× (5.72–6.45) | 2.76× (2.60–2.92) | 2.74× (2.57–2.93) | 1.08× (1.02–1.15) | 1.14× (1.07–1.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,026 | 3.8 | 0.51× (0.50–0.52) | 2.72× (2.57–2.87) | 5.34× (5.22–5.47) | 3.48× (3.44–3.52) | 5.71× (5.60–5.83) | 2.56× (2.49–2.63) | 2.55× (2.51–2.58) | 1.01× (0.98–1.03) | 1.05× (1.04–1.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.032 s | 609 | 6.2 | 2.96× (2.90–3.03) | 3.50× (3.42–3.58) | 3.93× (3.83–4.04) | 4.51× (4.41–4.60) | 4.98× (4.88–5.07) | 3.71× (3.64–3.78) | 3.01× (2.95–3.07) | 2.20× (2.07–2.34) | 2.23× (2.18–2.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 985 | 7.2 | 2.48× (2.33–2.64) | 3.03× (2.80–3.27) | 4.26× (3.88–4.67) | 3.99× (3.78–4.20) | 5.38× (5.04–5.73) | 5.75× (5.39–6.13) | 4.67× (4.37–4.98) | 3.41× (3.14–3.71) | 3.45× (3.23–3.69) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,016 | 12.9 | 2.29× (2.19–2.40) | 2.24× (2.06–2.43) | 3.44× (3.23–3.67) | 2.76× (2.57–2.96) | 4.74× (4.51–4.98) | 6.15× (5.84–6.47) | 4.99× (4.73–5.26) | 3.65× (3.35–3.97) | 3.69× (3.50–3.89) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.123 s | 65 | 55.9 | 1.39× (1.37–1.40) | 2.90× (2.88–2.93) | 2.58× (2.55–2.61) | 2.68× (2.66–2.70) | 2.98× (2.96–3.00) | 3.30× (3.28–3.32) | 3.37× (3.36–3.39) | 1.32× (1.31–1.33) | 1.32× (1.31–1.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.072 s | 111 | 60.6 | 1.35× (1.31–1.40) | 2.83× (2.76–2.90) | 2.70× (2.65–2.74) | 3.50× (3.39–3.61) | 3.17× (3.09–3.26) | 5.60× (5.45–5.76) | 5.72× (5.58–5.87) | 2.25× (2.19–2.30) | 2.24× (2.18–2.29) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 s | 138 | 80.1 | 1.50× (1.46–1.54) | 2.69× (2.59–2.80) | 2.62× (2.52–2.71) | 3.28× (3.22–3.35) | 3.27× (3.21–3.33) | 7.01× (6.88–7.15) | 7.16× (7.02–7.31) | 2.81× (2.75–2.87) | 2.80× (2.74–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.136 s | 71 | 51.4 | 1.44× (1.43–1.45) | 3.31× (3.27–3.35) | 2.78× (2.75–2.80) | 2.91× (2.89–2.92) | 3.25× (3.23–3.28) | 4.00× (3.98–4.03) | 4.21× (4.18–4.23) | 1.56× (1.55–1.57) | 1.56× (1.56–1.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.077 s | 125 | 55.9 | 1.41× (1.37–1.46) | 3.24× (3.12–3.36) | 2.89× (2.80–2.99) | 4.00× (3.89–4.11) | 3.41× (3.30–3.52) | 6.98× (6.76–7.21) | 7.34× (7.10–7.58) | 2.72× (2.64–2.81) | 2.73× (2.64–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 149 | 77.2 | 1.60× (1.57–1.63) | 3.07× (2.96–3.19) | 2.79× (2.73–2.84) | 3.79× (3.70–3.88) | 3.40× (3.35–3.45) | 8.53× (8.38–8.68) | 8.97× (8.79–9.14) | 3.32× (3.26–3.39) | 3.33× (3.27–3.39) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 844 | 4.8 | 0.62× (0.60–0.64) | 3.14× (2.83–3.48) | 4.45× (4.25–4.66) | 3.25× (3.13–3.38) | 6.90× (6.62–7.19) | 2.40× (2.31–2.50) | 2.42× (2.34–2.52) | 1.04× (1.00–1.08) | 0.90× (0.87–0.94) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 763 | 5.3 | 0.50× (0.48–0.52) | 2.38× (2.30–2.47) | 4.79× (4.62–4.97) | 2.97× (2.86–3.07) | 6.26× (6.05–6.48) | 2.16× (2.08–2.23) | 2.18× (2.11–2.25) | 0.93× (0.90–0.97) | 0.81× (0.78–0.84) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 778 | 5.2 | 0.52× (0.51–0.54) | 2.48× (2.33–2.65) | 4.84× (4.70–4.98) | 3.01× (2.93–3.10) | 6.27× (6.08–6.47) | 2.18× (2.12–2.23) | 2.20× (2.14–2.26) | 0.94× (0.92–0.97) | 0.82× (0.80–0.84) | yes |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 1 | 0.065 s | 226 | 16.3 | 1.94× (1.93–1.96) | 4.14× (4.08–4.20) | 4.68× (4.62–4.74) | 4.83× (4.79–4.87) | 4.90× (4.86–4.94) | 5.09× (5.05–5.12) | 4.99× (4.95–5.03) | 1.61× (1.59–1.62) | 1.57× (1.56–1.59) | yes |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 2 | 0.041 s | 353 | 19.6 | 1.64× (1.59–1.69) | 3.62× (3.51–3.74) | 4.27× (4.11–4.44) | 4.96× (4.80–5.13) | 4.64× (4.51–4.77) | 7.82× (7.62–8.02) | 7.67× (7.51–7.84) | 2.47× (2.41–2.53) | 2.42× (2.36–2.48) | yes |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 4 | 0.035 s | 421 | 29.0 | 1.75× (1.63–1.88) | 3.13× (2.85–3.45) | 3.70× (3.48–3.94) | 4.41× (4.15–4.68) | 4.49× (4.23–4.77) | 9.25× (8.71–9.82) | 9.08× (8.58–9.61) | 2.92× (2.75–3.10) | 2.86× (2.70–3.04) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.37× (3.30–3.44) | 7.49× (7.34–7.66) | 7.90× (7.74–8.06) | 8.94× (8.76–9.13) | 8.04× (7.86–8.23) | 8.55× (8.38–8.73) | 8.22× (8.05–8.40) | 2.63× (2.58–2.69) | 2.39× (2.34–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.33× (3.30–3.35) | 7.43× (7.38–7.49) | 8.14× (8.08–8.20) | 9.08× (8.99–9.17) | 7.78× (7.72–7.85) | 8.11× (8.06–8.15) | 7.79× (7.73–7.86) | 2.50× (2.47–2.52) | 2.27× (2.24–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.59× (3.51–3.66) | 6.93× (6.80–7.06) | 7.07× (6.92–7.21) | 7.84× (7.70–7.99) | 7.04× (6.89–7.20) | 5.21× (5.12–5.31) | 5.01× (4.92–5.11) | 1.61× (1.58–1.63) | 1.46× (1.43–1.49) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.59–0.65) | 3.00× (2.82–3.18) | 4.07× (3.84–4.32) | 3.67× (3.47–3.87) | 6.12× (5.83–6.42) | 2.79× (2.65–2.94) | 2.77× (2.62–2.93) | 1.14× (1.07–1.20) | 0.94× (0.90–1.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.54–0.60) | 2.77× (2.64–2.92) | 5.04× (4.77–5.32) | 3.43× (3.27–3.60) | 5.58× (5.29–5.89) | 2.56× (2.43–2.71) | 2.55× (2.40–2.71) | 1.05× (0.99–1.11) | 0.87× (0.82–0.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.53× (0.52–0.54) | 3.09× (3.03–3.14) | 4.77× (4.70–4.84) | 3.29× (3.25–3.33) | 5.31× (5.20–5.42) | 2.42× (2.35–2.49) | 2.41× (2.37–2.44) | 0.99× (0.96–1.01) | 0.82× (0.81–0.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.88× (2.82–2.95) | 3.20× (3.13–3.28) | 3.71× (3.63–3.79) | 4.36× (4.26–4.46) | 4.81× (4.71–4.91) | 3.59× (3.53–3.66) | 2.92× (2.86–2.99) | 2.15× (2.02–2.28) | 2.01× (1.98–2.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.60× (2.50–2.69) | 2.87× (2.77–2.98) | 3.42× (3.29–3.55) | 3.78× (3.64–3.92) | 4.15× (4.02–4.30) | 3.04× (2.94–3.14) | 2.47× (2.39–2.56) | 1.82× (1.71–1.93) | 1.70× (1.64–1.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.41× (2.32–2.51) | 2.01× (1.94–2.08) | 2.47× (2.38–2.57) | 2.42× (2.31–2.53) | 2.58× (2.49–2.68) | 1.73× (1.67–1.80) | 1.41× (1.36–1.47) | 1.04× (0.97–1.11) | 0.97× (0.93–1.01) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.37–1.40) | 2.82× (2.79–2.86) | 2.53× (2.50–2.55) | 2.66× (2.64–2.68) | 2.95× (2.93–2.98) | 3.28× (3.25–3.30) | 3.35× (3.33–3.37) | 1.32× (1.31–1.33) | 1.28× (1.28–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.33× (1.31–1.35) | 2.76× (2.71–2.80) | 2.44× (2.40–2.48) | 3.24× (3.19–3.29) | 2.81× (2.76–2.85) | 3.00× (2.95–3.06) | 3.07× (3.02–3.11) | 1.21× (1.19–1.23) | 1.18× (1.16–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.59–1.68) | 2.87× (2.77–2.96) | 2.41× (2.36–2.47) | 3.21× (3.14–3.27) | 2.85× (2.79–2.92) | 2.27× (2.22–2.32) | 2.31× (2.27–2.36) | 0.91× (0.89–0.93) | 0.89× (0.87–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.43× (1.42–1.44) | 3.26× (3.22–3.29) | 2.72× (2.69–2.75) | 2.88× (2.87–2.90) | 3.23× (3.21–3.25) | 3.97× (3.95–4.00) | 4.18× (4.15–4.20) | 1.56× (1.55–1.56) | 1.54× (1.53–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.34–1.43) | 3.14× (3.06–3.23) | 2.64× (2.56–2.71) | 3.67× (3.56–3.78) | 3.06× (2.98–3.14) | 3.62× (3.52–3.72) | 3.80× (3.70–3.91) | 1.42× (1.38–1.46) | 1.40× (1.36–1.44) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.72× (1.68–1.77) | 3.29× (3.21–3.38) | 2.42× (2.33–2.50) | 3.43× (3.33–3.54) | 3.01× (2.93–3.09) | 2.63× (2.57–2.69) | 2.77× (2.70–2.83) | 1.03× (1.01–1.06) | 1.02× (1.00–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.64–0.68) | 2.49× (2.38–2.61) | 3.52× (3.40–3.65) | 3.02× (2.92–3.13) | 6.30× (6.03–6.57) | 2.26× (2.18–2.35) | 2.28× (2.20–2.36) | 1.04× (1.00–1.07) | 0.84× (0.81–0.88) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.53–0.57) | 2.16× (2.10–2.23) | 3.98× (3.86–4.10) | 2.74× (2.67–2.83) | 5.57× (5.42–5.74) | 1.98× (1.92–2.03) | 1.99× (1.94–2.05) | 0.91× (0.88–0.93) | 0.74× (0.72–0.76) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.56× (0.54–0.58) | 2.61× (2.53–2.70) | 4.08× (3.97–4.19) | 2.84× (2.76–2.91) | 5.68× (5.51–5.85) | 2.03× (1.98–2.08) | 2.04× (1.99–2.09) | 0.93× (0.90–0.96) | 0.76× (0.74–0.78) |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 1 | 1.92× (1.91–1.94) | 4.00× (3.97–4.03) | 4.51× (4.47–4.55) | 4.74× (4.70–4.79) | 4.81× (4.77–4.85) | 5.00× (4.96–5.03) | 4.91× (4.86–4.95) | 1.59× (1.58–1.61) | 1.50× (1.49–1.51) |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 2 | 1.68× (1.66–1.71) | 3.51× (3.44–3.58) | 4.07× (4.00–4.14) | 4.47× (4.40–4.55) | 4.12× (4.05–4.19) | 4.12× (4.05–4.19) | 4.04× (3.99–4.10) | 1.31× (1.29–1.33) | 1.24× (1.22–1.26) |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 4 | 1.91× (1.85–1.98) | 3.17× (3.06–3.29) | 3.61× (3.49–3.72) | 3.93× (3.80–4.05) | 3.63× (3.53–3.73) | 2.81× (2.72–2.90) | 2.76× (2.68–2.84) | 0.90× (0.87–0.92) | 0.84× (0.82–0.87) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.26× (0.25–0.27) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.98× (10.96–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.35× (0.34–0.36) | 0.50× (0.44–0.58) | 0.71× (0.69–0.74) | 0.60× (0.58–0.62) | 5.71× (5.54–5.88) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.36× (0.34–0.38) | 0.50× (0.43–0.59) | 0.64× (0.59–0.68) | 0.56× (0.52–0.59) | 3.73× (3.51–3.97) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.28× (4.26–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.44) | 1.68× (1.68–1.68) | 0.65× (0.65–0.66) | 7.94× (7.92–7.95) | 0.65× (0.65–0.65) | 0.68× (0.68–0.68) | 0.74× (0.74–0.74) | 0.36× (0.36–0.36) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.73× (0.71–0.74) | 0.67× (0.61–0.73) | 1.73× (1.70–1.76) | 0.86× (0.85–0.88) | 6.26× (6.15–6.38) | 0.51× (0.50–0.52) | 0.53× (0.52–0.54) | 0.58× (0.57–0.59) | 0.28× (0.28–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.68–0.71) | 0.75× (0.69–0.80) | 1.24× (1.20–1.28) | 0.77× (0.75–0.78) | 3.70× (3.62–3.79) | 0.29× (0.29–0.30) | 0.30× (0.30–0.31) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.15–1.17) | 0.72× (0.72–0.73) | 5.14× (5.12–5.16) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.50× (0.48–0.51) | 0.40× (0.37–0.43) | 0.89× (0.87–0.92) | 0.69× (0.65–0.72) | 3.09× (3.00–3.19) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.81× (0.79–0.83) | 2.51× (2.45–2.57) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.38× (0.36–0.40) | 0.84× (0.82–0.87) | 0.70× (0.67–0.72) | 2.89× (2.81–2.98) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.48× (0.45–0.51) | 0.80× (0.78–0.81) | 0.80× (0.78–0.83) | 2.31× (2.26–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.06× (1.05–1.06) | 0.75× (0.75–0.76) | 5.31× (5.29–5.32) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.29–0.30) | 0.24× (0.24–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.23) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.19× (6.17–6.22) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.39) | 0.66× (0.66–0.67) | 6.38× (6.37–6.38) | 0.39× (0.38–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 2 | 36 MB | 0.47× (0.45–0.48) | 0.43× (0.38–0.47) | 1.08× (1.06–1.11) | 0.70× (0.69–0.72) | 3.45× (3.36–3.54) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.34× (0.33–0.35) | 0.17× (0.16–0.17) |
| `text` (2.53 MB → 14.6 MB, 5.8×) | 4 | 53 MB | 0.53× (0.51–0.56) | 0.47× (0.43–0.51) | 0.97× (0.93–1.01) | 0.81× (0.76–0.85) | 2.49× (2.39–2.60) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.23× (0.22–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 504 | 1.99 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 901 | 2.12 | 1.06× (1.04–1.07) | 89% | 1.05× (1.04–1.06) | 1.04× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,119 | 3.32 | 1.64× (1.61–1.68) | 56% | 1.52× (1.51–1.53) | 1.74× (1.73–1.76) |

**Best:** mibz2 at 4 threads (1,119 MB/s); lbzip2 at 4 threads (172 MB/s); pbz2 at 4 threads (302 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.08, lbzip2 1t 0.04, lbzip2 2t 0.21, lbzip2 4t 0.26, pbz2 1t 0.00, pbz2 2t 0.38, pbz2 4t 0.38. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.53 | 0.02 | 0.08 | 0.187 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3595 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 58.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.94 | 9.3% |
| MTF + RLE2 | 1.63 | 16.1% |
| T-vector build | 1.60 | 15.8% |
| walk (the one each build uses) | 5.55 | 54.9% |
| of which: segment assembly | 0.28 | 2.8% |
| RLE1 | 0.26 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.12 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.434 | 1.134 | (in its total) | 2.61× (2.61–2.62) |  |
| walk | 1.478 | 2.914 | (in its total) | 1.98× (1.97–1.99) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.079 | 0.073 | (in its total) | 0.94× (0.93–0.94) |  |
| inverse BWT, total | 1.991 | 4.123 | 2.141 | 2.08× (2.07–2.08) | 1.08× (1.07–1.08) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h01m ± 7m27s |
| 4 threads | 27m20s ± 3m51s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m19s ± 2m46s |
| the scanner alone (its serial part) | 8m53s ± 19.7s |
| the workers alone, over 4 | 19m19s ± 2m46s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.53 ns a byte of output at 4 threads (CPU, summed over them), 1.71 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,531,345 | 14,565,598 | \`6b3f135b0f0d9dd8…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3595.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
