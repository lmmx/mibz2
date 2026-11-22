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

The latest record: [`b1faadc`](<https://github.com/lmmx/mibz2/commit/b1faadccab1b36ed91e2771af96a1d93c4f9e4b1>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45074172721>)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.389 s | 539 | 7.3 | 3.48× (3.44–3.53) | 7.98× (7.88–8.08) | 8.57× (8.48–8.65) | 9.44× (9.32–9.57) | 8.49× (8.40–8.59) | 8.94× (8.83–9.05) | 8.66× (8.56–8.76) | 2.76× (2.73–2.79) | 2.43× (2.40–2.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.233 s | 900 | 8.5 | 3.20× (3.07–3.34) | 6.93× (6.58–7.29) | 8.32× (8.02–8.64) | 8.28× (7.97–8.61) | 7.49× (7.18–7.81) | 14.72× (14.13–15.35) | 14.26× (13.71–14.83) | 4.54× (4.36–4.73) | 4.01× (3.85–4.17) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.192 s | 1,094 | 13.0 | 3.35× (3.30–3.40) | 6.11× (6.03–6.20) | 7.32× (7.17–7.48) | 7.19× (7.06–7.33) | 6.96× (6.85–7.07) | 18.09× (17.75–18.45) | 17.52× (17.19–17.86) | 5.58× (5.47–5.68) | 4.92× (4.84–5.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.017 s | 1,180 | 3.6 | 0.60× (0.59–0.62) | 3.04× (2.95–3.13) | 4.33× (4.20–4.47) | 3.62× (3.53–3.71) | 6.29× (6.14–6.43) | 2.75× (2.67–2.84) | 2.80× (2.72–2.88) | 1.11× (1.09–1.14) | 1.23× (1.19–1.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,133 | 3.9 | 0.51× (0.50–0.53) | 2.87× (2.79–2.96) | 5.80× (5.62–5.99) | 3.46× (3.36–3.56) | 5.87× (5.74–6.01) | 2.60× (2.53–2.68) | 2.64× (2.57–2.72) | 1.05× (1.03–1.07) | 1.16× (1.12–1.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,127 | 3.8 | 0.51× (0.50–0.53) | 2.84× (2.76–2.93) | 5.72× (5.55–5.90) | 3.42× (3.32–3.52) | 5.84× (5.72–5.96) | 2.57× (2.50–2.64) | 2.61× (2.54–2.68) | 1.04× (1.01–1.07) | 1.15× (1.11–1.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 s | 632 | 6.5 | 2.62× (2.59–2.66) | 3.31× (3.25–3.36) | 4.00× (3.93–4.08) | 4.30× (4.24–4.36) | 4.74× (4.69–4.79) | 3.18× (3.13–3.23) | 2.93× (2.89–2.97) | 1.99× (1.97–2.00) | 2.02× (1.94–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 s | 998 | 7.8 | 2.25× (2.02–2.49) | 3.15× (2.85–3.48) | 4.18× (3.74–4.68) | 3.70× (3.35–4.08) | 5.07× (4.60–5.58) | 4.81× (4.34–5.34) | 4.44× (4.01–4.90) | 3.00× (2.73–3.31) | 3.06× (2.75–3.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,013 | 13.4 | 2.05× (1.98–2.13) | 2.20× (2.13–2.27) | 3.30× (3.14–3.47) | 2.66× (2.49–2.84) | 4.50× (4.36–4.63) | 5.13× (4.97–5.29) | 4.73× (4.58–4.88) | 3.20× (3.08–3.32) | 3.26× (3.09–3.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.112 s | 72 | 55.4 | 1.39× (1.39–1.40) | 3.05× (3.02–3.08) | 2.80× (2.76–2.83) | 2.85× (2.83–2.87) | 3.13× (3.12–3.15) | 3.48× (3.46–3.50) | 3.54× (3.51–3.56) | 1.28× (1.28–1.29) | 1.27× (1.26–1.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.065 s | 124 | 59.6 | 1.37× (1.32–1.41) | 3.05× (2.89–3.22) | 2.98× (2.89–3.07) | 3.60× (3.48–3.72) | 3.37× (3.27–3.48) | 5.98× (5.79–6.18) | 6.08× (5.90–6.27) | 2.21× (2.13–2.28) | 2.18× (2.11–2.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 152 | 81.2 | 1.51× (1.48–1.55) | 2.97× (2.87–3.07) | 2.84× (2.77–2.91) | 3.48× (3.45–3.52) | 3.49× (3.43–3.55) | 7.41× (7.30–7.52) | 7.54× (7.42–7.66) | 2.74× (2.69–2.78) | 2.70× (2.65–2.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.125 s | 77 | 51.6 | 1.39× (1.31–1.47) | 3.31× (3.13–3.50) | 2.86× (2.70–3.04) | 2.93× (2.77–3.10) | 3.24× (3.06–3.42) | 3.99× (3.77–4.21) | 4.22× (3.99–4.45) | 1.44× (1.37–1.53) | 1.49× (1.41–1.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 133 | 55.5 | 1.40× (1.31–1.49) | 3.26× (3.16–3.36) | 2.99× (2.91–3.08) | 3.99× (3.88–4.11) | 3.44× (3.35–3.53) | 7.10× (6.94–7.27) | 7.51× (7.33–7.70) | 2.57× (2.51–2.63) | 2.66× (2.60–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.060 s | 161 | 79.7 | 1.53× (1.50–1.56) | 3.12× (3.04–3.21) | 2.86× (2.80–2.92) | 3.80× (3.72–3.87) | 3.37× (3.30–3.43) | 8.54× (8.40–8.69) | 9.04× (8.86–9.22) | 3.09× (3.04–3.14) | 3.20× (3.14–3.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 905 | 4.9 | 0.67× (0.65–0.69) | 2.79× (2.71–2.88) | 3.93× (3.77–4.11) | 3.13× (3.04–3.22) | 6.88× (6.71–7.05) | 2.40× (2.34–2.46) | 2.44× (2.38–2.50) | 1.01× (0.98–1.04) | 0.95× (0.92–0.98) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 816 | 5.7 | 0.49× (0.48–0.51) | 2.54× (2.47–2.60) | 5.16× (5.03–5.30) | 2.88× (2.82–2.93) | 6.26× (6.14–6.38) | 2.18× (2.14–2.22) | 2.21× (2.17–2.25) | 0.92× (0.90–0.93) | 0.86× (0.84–0.88) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 820 | 5.5 | 0.50× (0.48–0.51) | 2.51× (2.43–2.60) | 5.00× (4.79–5.22) | 2.87× (2.78–2.95) | 6.13× (5.99–6.29) | 2.15× (2.09–2.21) | 2.19× (2.13–2.25) | 0.91× (0.88–0.93) | 0.85× (0.82–0.88) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 0.061 s | 227 | 17.7 | 1.92× (1.91–1.93) | 4.33× (4.30–4.35) | 4.88× (4.83–4.94) | 4.95× (4.93–4.97) | 5.04× (5.02–5.06) | 5.37× (5.34–5.40) | 5.22× (5.20–5.24) | 1.57× (1.57–1.58) | 1.50× (1.49–1.51) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 0.039 s | 358 | 20.8 | 1.79× (1.52–2.10) | 3.77× (3.71–3.84) | 4.99× (4.82–5.15) | 5.06× (4.93–5.20) | 4.81× (4.71–4.92) | 8.44× (8.24–8.66) | 8.21× (8.03–8.40) | 2.48× (2.42–2.53) | 2.35× (2.30–2.41) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 0.035 s | 394 | 31.0 | 1.60× (1.54–1.67) | 3.30× (3.17–3.44) | 4.19× (4.07–4.32) | 4.26× (4.15–4.38) | 4.50× (4.39–4.61) | 9.33× (9.12–9.55) | 9.08× (8.89–9.27) | 2.74× (2.67–2.80) | 2.60× (2.55–2.65) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.47× (3.43–3.52) | 7.93× (7.83–8.03) | 8.52× (8.43–8.61) | 9.40× (9.28–9.53) | 8.46× (8.36–8.55) | 8.90× (8.79–9.01) | 8.62× (8.52–8.72) | 2.75× (2.72–2.78) | 2.42× (2.39–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.09× (3.04–3.13) | 7.08× (6.95–7.22) | 7.85× (7.74–7.96) | 8.54× (8.42–8.67) | 7.37× (7.24–7.50) | 7.63× (7.51–7.75) | 7.39× (7.28–7.49) | 2.35× (2.32–2.39) | 2.07× (2.04–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.18× (3.11–3.26) | 6.48× (6.35–6.60) | 7.03× (6.92–7.14) | 7.64× (7.53–7.75) | 6.74× (6.62–6.86) | 4.90× (4.80–4.99) | 4.74× (4.66–4.83) | 1.51× (1.48–1.54) | 1.33× (1.30–1.36) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 2.80× (2.71–2.89) | 3.80× (3.70–3.89) | 3.37× (3.29–3.46) | 5.80× (5.67–5.93) | 2.59× (2.52–2.67) | 2.63× (2.56–2.71) | 1.11× (1.08–1.14) | 0.79× (0.77–0.82) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.57) | 2.62× (2.53–2.70) | 5.09× (4.96–5.21) | 3.16× (3.07–3.26) | 5.30× (5.18–5.43) | 2.39× (2.32–2.46) | 2.43× (2.36–2.50) | 1.02× (1.00–1.04) | 0.73× (0.71–0.76) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.53–0.56) | 2.93× (2.83–3.03) | 5.13× (5.01–5.26) | 3.20× (3.10–3.29) | 5.36× (5.25–5.47) | 2.41× (2.34–2.48) | 2.44× (2.38–2.51) | 1.03× (1.00–1.05) | 0.74× (0.72–0.76) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.54× (2.50–2.58) | 3.06× (3.02–3.11) | 3.61× (3.56–3.66) | 4.12× (4.06–4.18) | 4.54× (4.49–4.59) | 3.07× (3.02–3.11) | 2.83× (2.79–2.87) | 1.93× (1.91–1.95) | 1.89× (1.85–1.93) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.27–2.42) | 2.78× (2.69–2.88) | 3.39× (3.30–3.47) | 3.57× (3.45–3.70) | 3.94× (3.82–4.07) | 2.58× (2.49–2.67) | 2.38× (2.30–2.47) | 1.62× (1.58–1.67) | 1.59× (1.53–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.13× (2.05–2.21) | 1.90× (1.82–1.99) | 2.39× (2.32–2.47) | 2.32× (2.24–2.41) | 2.47× (2.40–2.55) | 1.47× (1.42–1.52) | 1.36× (1.31–1.40) | 0.92× (0.90–0.95) | 0.91× (0.87–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.38–1.39) | 2.97× (2.96–2.99) | 2.73× (2.71–2.75) | 2.82× (2.81–2.84) | 3.10× (3.09–3.11) | 3.44× (3.42–3.46) | 3.50× (3.48–3.52) | 1.28× (1.28–1.28) | 1.25× (1.24–1.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.36× (1.34–1.37) | 2.94× (2.92–2.97) | 2.70× (2.66–2.73) | 3.40× (3.37–3.43) | 2.98× (2.96–3.01) | 3.20× (3.17–3.24) | 3.26× (3.23–3.30) | 1.19× (1.18–1.21) | 1.16× (1.15–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.62× (1.58–1.66) | 2.96× (2.84–3.09) | 2.60× (2.55–2.66) | 3.36× (3.30–3.42) | 3.02× (2.98–3.07) | 2.37× (2.34–2.40) | 2.41× (2.38–2.45) | 0.88× (0.87–0.90) | 0.86× (0.85–0.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.38× (1.31–1.46) | 3.23× (3.06–3.42) | 2.79× (2.64–2.95) | 2.90× (2.74–3.06) | 3.20× (3.03–3.38) | 3.94× (3.74–4.16) | 4.17× (3.95–4.40) | 1.44× (1.36–1.52) | 1.43× (1.35–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.34–1.41) | 3.23× (3.15–3.31) | 2.78× (2.72–2.85) | 3.72× (3.64–3.81) | 3.13× (3.06–3.20) | 3.74× (3.66–3.82) | 3.95× (3.86–4.05) | 1.36× (1.33–1.39) | 1.35× (1.32–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.56–1.67) | 3.27× (3.22–3.33) | 2.50× (2.46–2.54) | 3.43× (3.32–3.54) | 2.98× (2.91–3.06) | 2.62× (2.57–2.68) | 2.77× (2.71–2.84) | 0.96× (0.93–0.98) | 0.95× (0.93–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.70–0.73) | 2.44× (2.36–2.52) | 3.41× (3.32–3.50) | 2.86× (2.78–2.95) | 6.14× (5.98–6.30) | 2.22× (2.16–2.29) | 2.26× (2.20–2.32) | 1.01× (0.99–1.04) | 0.75× (0.73–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.11× (2.04–2.19) | 4.08× (4.02–4.15) | 2.58× (2.53–2.64) | 5.40× (5.29–5.51) | 1.94× (1.90–1.99) | 1.98× (1.94–2.02) | 0.88× (0.87–0.90) | 0.66× (0.64–0.68) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.53–0.56) | 2.44× (2.31–2.58) | 4.09× (3.98–4.20) | 2.64× (2.57–2.72) | 5.45× (5.32–5.59) | 1.98× (1.93–2.03) | 2.01× (1.96–2.07) | 0.90× (0.87–0.92) | 0.67× (0.65–0.69) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 1.90× (1.89–1.91) | 4.18× (4.16–4.21) | 4.73× (4.69–4.76) | 4.85× (4.82–4.87) | 4.93× (4.90–4.95) | 5.25× (5.22–5.29) | 5.11× (5.08–5.13) | 1.56× (1.55–1.57) | 1.44× (1.43–1.45) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 1.70× (1.69–1.72) | 3.76× (3.70–3.83) | 4.44× (4.39–4.48) | 4.70× (4.66–4.75) | 4.29× (4.24–4.35) | 4.46× (4.41–4.51) | 4.34× (4.29–4.39) | 1.32× (1.31–1.34) | 1.23× (1.21–1.24) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 1.84× (1.77–1.92) | 3.39× (3.25–3.54) | 3.80× (3.69–3.92) | 4.19× (4.06–4.32) | 3.83× (3.71–3.95) | 2.97× (2.88–3.06) | 2.89× (2.81–2.96) | 0.88× (0.86–0.91) | 0.82× (0.79–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.28× (0.26–0.30) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.00× (10.98–11.02) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 53 MB | 0.35× (0.34–0.36) | 0.51× (0.46–0.55) | 0.73× (0.70–0.75) | 0.62× (0.60–0.64) | 5.81× (5.62–6.02) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.36–0.39) | 0.51× (0.44–0.59) | 0.66× (0.63–0.69) | 0.60× (0.57–0.62) | 3.89× (3.71–4.08) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.16) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.02) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.17× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.93–0.93) | 4.84× (4.83–4.85) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.02× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.43× (0.42–0.44) | 1.69× (1.68–1.70) | 0.65× (0.65–0.66) | 7.96× (7.92–8.00) | 0.65× (0.65–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 21 MB | 0.71× (0.68–0.74) | 0.69× (0.63–0.76) | 1.69× (1.63–1.75) | 0.84× (0.81–0.87) | 6.09× (5.88–6.31) | 0.50× (0.48–0.51) | 0.52× (0.50–0.54) | 0.56× (0.54–0.59) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.69× (0.66–0.71) | 0.73× (0.67–0.80) | 1.23× (1.18–1.29) | 0.76× (0.73–0.80) | 3.66× (3.50–3.83) | 0.29× (0.28–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.31–0.32) | 1.16× (1.16–1.16) | 0.73× (0.72–0.73) | 5.14× (5.13–5.15) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.39× (0.36–0.43) | 0.91× (0.88–0.94) | 0.70× (0.67–0.73) | 3.11× (3.02–3.20) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.62–0.67) | 0.46× (0.45–0.48) | 0.88× (0.86–0.90) | 0.82× (0.79–0.84) | 2.57× (2.51–2.63) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.13× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.32) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.16–5.21) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.36× (0.33–0.39) | 0.85× (0.82–0.89) | 0.70× (0.67–0.73) | 2.91× (2.81–3.02) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.48) | 0.80× (0.79–0.82) | 0.83× (0.81–0.85) | 2.31× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.23) | 1.06× (1.06–1.07) | 0.76× (0.75–0.76) | 5.32× (5.29–5.34) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.28) | 0.24× (0.23–0.24) | 1.87× (1.87–1.88) | 0.90× (0.90–0.90) | 6.20× (6.18–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.29) | 0.23× (0.23–0.23) | 1.86× (1.85–1.87) | 0.90× (0.89–0.91) | 6.18× (6.15–6.20) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.67× (0.66–0.67) | 6.42× (6.38–6.45) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.63× (0.63–0.63) | 0.31× (0.30–0.31) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 33 MB | 0.51× (0.50–0.52) | 0.47× (0.43–0.52) | 1.17× (1.13–1.22) | 0.77× (0.75–0.80) | 3.78× (3.68–3.88) | 0.23× (0.22–0.24) | 0.25× (0.24–0.25) | 0.37× (0.36–0.38) | 0.18× (0.18–0.19) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 49 MB | 0.55× (0.53–0.58) | 0.55× (0.50–0.60) | 1.02× (0.99–1.06) | 0.85× (0.82–0.88) | 2.60× (2.52–2.69) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 539 | 1.86 | 1 | – | 1 | 1 |
| 2: CPUs 0,1, one a core | 900 | 2.17 | 1.17× (1.16–1.18) | 82% | 1.04× (1.03–1.05) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,094 | 3.34 | 1.82× (1.79–1.85) | 51% | 1.48× (1.47–1.50) | 1.67× (1.64–1.70) |

**Best:** mibz2 at 4 threads (1,094 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (329 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.77, mibz2 4t 1.02, lbzip2 1t 0.04, lbzip2 2t 0.21, lbzip2 4t 0.30, pbz2 1t 0.00, pbz2 2t 0.28, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.55 | 0.02 | 0.08 | 0.192 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3901 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.9s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 9.3% |
| MTF + RLE2 | 1.72 | 15.9% |
| T-vector build | 1.60 | 14.8% |
| walk (the one each build uses) | 6.13 | 56.4% |
| of which: segment assembly | 0.32 | 2.9% |
| RLE1 | 0.26 | 2.4% |
| CRC | 0.14 | 1.3% |
| **total** | 10.87 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.369 | 0.934 | (in its total) | 2.52× (2.52–2.53) |  |
| walk | 1.454 | 2.769 | (in its total) | 1.90× (1.90–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.083 | 0.074 | (in its total) | 0.89× (0.88–0.90) |  |
| inverse BWT, total | 1.905 | 3.776 | 1.999 | 1.98× (1.97–1.99) | 1.05× (1.04–1.05) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m18s ± 6m42s |
| 4 threads | 27m50s ± 3m56s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m58s ± 2m51s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 19m58s ± 2m51s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.62 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,736,457 | 13,860,455 | \`4685e5a04aaa92a2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3901.4 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261116.88.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
