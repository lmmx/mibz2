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

The latest record: [`ba9acc8`](<https://github.com/lmmx/mibz2/commit/ba9acc834183bc7e93c2c7cae4fa28b86983bd5e>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42794909504>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.515 s | 407 | 6.4 | 2.36× (2.35–2.37) | 5.40× (5.34–5.45) | 5.59× (5.52–5.66) | 6.33× (6.26–6.39) | 5.59× (5.52–5.65) | 5.82× (5.78–5.87) | 5.55× (5.48–5.62) | 1.75× (1.73–1.76) | 1.95× (1.94–1.97) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.287 s | 731 | 6.8 | 2.40× (2.33–2.49) | 5.01× (4.76–5.26) | 5.76× (5.50–6.04) | 5.98× (5.75–6.23) | 5.27× (5.06–5.48) | 10.29× (9.91–10.69) | 9.80× (9.42–10.20) | 3.09× (2.97–3.21) | 3.45× (3.33–3.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.245 s | 855 | 11.3 | 2.38× (2.32–2.45) | 3.44× (3.34–3.55) | 4.24× (4.08–4.41) | 4.16× (4.04–4.28) | 4.11× (3.98–4.24) | 12.13× (11.80–12.46) | 11.55× (11.23–11.87) | 3.64× (3.54–3.74) | 4.07× (3.95–4.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,140 | 2.5 | 0.58× (0.57–0.58) | 3.48× (3.44–3.52) | 4.65× (4.60–4.70) | 3.63× (3.58–3.67) | 6.27× (5.94–6.61) | 2.70× (2.67–2.73) | 2.70× (2.67–2.74) | 0.98× (0.95–1.00) | 1.96× (1.77–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,114 | 2.6 | 0.53× (0.52–0.54) | 2.85× (2.78–2.91) | 5.70× (5.58–5.83) | 3.55× (3.48–3.63) | 5.93× (5.79–6.06) | 2.64× (2.58–2.69) | 2.64× (2.58–2.70) | 0.95× (0.93–0.98) | 1.91× (1.72–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,102 | 2.6 | 0.53× (0.52–0.54) | 2.83× (2.78–2.87) | 5.65× (5.55–5.74) | 3.52× (3.47–3.57) | 5.93× (5.83–6.03) | 2.61× (2.57–2.65) | 2.61× (2.57–2.66) | 0.94× (0.92–0.97) | 1.89× (1.69–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 s | 709 | 3.9 | 2.59× (2.57–2.61) | 2.95× (2.93–2.97) | 3.72× (3.54–3.91) | 3.96× (3.87–4.06) | 4.26× (4.19–4.32) | 2.66× (2.64–2.68) | 2.17× (2.16–2.18) | 2.04× (1.94–2.14) | 2.93× (2.91–2.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,107 | 4.6 | 2.05× (1.75–2.40) | 2.69× (2.27–3.18) | 3.93× (3.34–4.63) | 3.36× (2.83–3.98) | 4.60× (3.98–5.31) | 3.78× (3.25–4.40) | 3.09× (2.65–3.60) | 2.90× (2.45–3.44) | 4.17× (3.58–4.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 956 | 9.3 | 1.82× (1.76–1.88) | 1.90× (1.74–2.07) | 2.96× (2.62–3.35) | 2.17× (2.06–2.29) | 3.93× (3.85–4.01) | 3.58× (3.51–3.64) | 2.92× (2.87–2.98) | 2.74× (2.59–2.90) | 3.94× (3.87–4.02) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 36.3 | 1.38× (1.37–1.38) | 2.63× (2.60–2.67) | 2.39× (2.38–2.40) | 2.51× (2.49–2.53) | 2.78× (2.76–2.81) | 2.97× (2.95–2.98) | 3.08× (3.05–3.11) | 1.24× (1.24–1.25) | 1.40× (1.37–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.066 s | 121 | 40.6 | 1.28× (1.23–1.33) | 2.59× (2.43–2.75) | 2.37× (2.28–2.46) | 3.15× (3.05–3.25) | 2.87× (2.76–2.98) | 4.80× (4.62–4.99) | 4.98× (4.78–5.19) | 2.01× (1.93–2.10) | 2.27× (2.17–2.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 146 | 53.8 | 1.34× (1.31–1.38) | 2.13× (2.06–2.21) | 2.14× (2.08–2.20) | 2.69× (2.61–2.77) | 2.92× (2.83–3.01) | 5.97× (5.82–6.12) | 6.19× (6.02–6.36) | 2.50× (2.44–2.57) | 2.82× (2.74–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 32.4 | 1.53× (1.53–1.54) | 3.07× (3.04–3.10) | 2.70× (2.66–2.73) | 2.82× (2.80–2.85) | 3.15× (3.13–3.17) | 3.72× (3.71–3.73) | 4.00× (3.97–4.03) | 1.50× (1.50–1.51) | 1.80× (1.80–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 131 | 37.1 | 1.37× (1.33–1.40) | 2.81× (2.60–3.04) | 2.52× (2.44–2.61) | 3.52× (3.43–3.62) | 3.05× (2.95–3.16) | 5.98× (5.79–6.17) | 6.43× (6.21–6.67) | 2.42× (2.34–2.49) | 2.89× (2.80–2.99) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 155 | 51.7 | 1.42× (1.39–1.45) | 2.28× (2.22–2.35) | 2.27× (2.20–2.35) | 3.11× (3.03–3.19) | 2.91× (2.81–3.01) | 7.05× (6.91–7.19) | 7.58× (7.40–7.77) | 2.85× (2.79–2.90) | 3.41× (3.34–3.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 855 | 3.5 | 0.62× (0.62–0.63) | 3.48× (3.44–3.52) | 4.38× (4.33–4.43) | 3.12× (3.08–3.16) | 6.70× (6.60–6.81) | 2.29× (2.26–2.31) | 2.32× (2.30–2.35) | 0.95× (0.93–0.97) | 1.75× (1.73–1.77) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 823 | 3.7 | 0.53× (0.52–0.54) | 2.54× (2.50–2.59) | 5.11× (5.02–5.20) | 3.03× (2.97–3.09) | 6.40× (6.25–6.55) | 2.22× (2.19–2.25) | 2.25× (2.21–2.29) | 0.92× (0.90–0.94) | 1.69× (1.66–1.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 810 | 3.6 | 0.53× (0.52–0.54) | 2.50× (2.47–2.54) | 5.02× (4.95–5.09) | 2.99× (2.95–3.03) | 6.33× (6.20–6.47) | 2.18× (2.14–2.21) | 2.21× (2.18–2.24) | 0.91× (0.89–0.92) | 1.66× (1.64–1.69) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.073 s | 169 | 15.8 | 1.34× (1.33–1.36) | 2.90× (2.85–2.94) | 3.17× (3.12–3.21) | 3.37× (3.32–3.42) | 3.53× (3.47–3.58) | 3.52× (3.47–3.56) | 3.47× (3.40–3.54) | 1.11× (1.10–1.13) | 1.24× (1.22–1.26) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.047 s | 263 | 18.7 | 1.10× (1.05–1.16) | 2.57× (2.40–2.76) | 2.86× (2.74–2.99) | 3.51× (3.31–3.73) | 3.32× (3.15–3.49) | 5.36× (5.07–5.67) | 5.29× (5.00–5.60) | 1.70× (1.61–1.79) | 1.89× (1.79–1.99) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.040 s | 312 | 26.6 | 1.23× (1.20–1.26) | 2.07× (1.95–2.19) | 2.54× (2.41–2.67) | 3.05× (2.93–3.17) | 3.19× (3.11–3.27) | 6.58× (6.40–6.77) | 6.49× (6.33–6.66) | 2.09× (2.04–2.14) | 2.32× (2.27–2.37) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.35× (2.34–2.36) | 5.37× (5.32–5.43) | 5.57× (5.50–5.63) | 6.31× (6.25–6.37) | 5.57× (5.51–5.64) | 5.81× (5.76–5.86) | 5.53× (5.46–5.60) | 1.74× (1.73–1.76) | 1.94× (1.93–1.96) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.20–2.29) | 5.09× (4.98–5.20) | 5.33× (5.23–5.43) | 6.20× (6.08–6.31) | 5.20× (5.10–5.31) | 5.39× (5.29–5.49) | 5.13× (5.02–5.24) | 1.62× (1.59–1.65) | 1.80× (1.77–1.84) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.20× (2.14–2.26) | 3.60× (3.49–3.71) | 3.87× (3.73–4.01) | 4.36× (4.21–4.51) | 3.77× (3.65–3.89) | 3.25× (3.14–3.36) | 3.09× (2.99–3.20) | 0.98× (0.94–1.01) | 1.09× (1.05–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.96× (2.93–3.00) | 3.94× (3.90–3.98) | 3.39× (3.36–3.43) | 5.80× (5.51–6.10) | 2.55× (2.52–2.57) | 2.55× (2.52–2.58) | 0.98× (0.96–1.00) | 1.66× (1.65–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.74× (2.69–2.80) | 4.95× (4.86–5.05) | 3.30× (3.24–3.37) | 5.43× (5.31–5.56) | 2.46× (2.41–2.51) | 2.46× (2.41–2.51) | 0.95× (0.93–0.96) | 1.60× (1.57–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.02× (2.98–3.07) | 4.97× (4.88–5.06) | 3.34× (3.30–3.39) | 5.52× (5.42–5.63) | 2.48× (2.44–2.52) | 2.49× (2.44–2.53) | 0.95× (0.93–0.98) | 1.62× (1.59–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.50× (2.48–2.52) | 2.72× (2.71–2.74) | 3.31× (3.13–3.50) | 3.79× (3.70–3.87) | 4.07× (4.00–4.13) | 2.56× (2.54–2.57) | 2.10× (2.09–2.11) | 1.98× (1.88–2.08) | 2.53× (2.51–2.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.20–2.29) | 2.48× (2.42–2.55) | 3.29× (3.22–3.36) | 3.33× (3.23–3.42) | 3.53× (3.48–3.59) | 2.14× (2.10–2.19) | 1.76× (1.72–1.80) | 1.66× (1.56–1.76) | 2.12× (2.08–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.97× (1.89–2.06) | 1.66× (1.57–1.76) | 2.26× (2.05–2.50) | 2.01× (1.91–2.12) | 2.06× (1.98–2.15) | 1.08× (1.04–1.12) | 0.89× (0.85–0.92) | 0.83× (0.78–0.89) | 1.07× (1.02–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.56× (2.55–2.57) | 2.33× (2.32–2.34) | 2.49× (2.47–2.50) | 2.75× (2.73–2.78) | 2.94× (2.92–2.95) | 3.05× (3.02–3.07) | 1.24× (1.23–1.25) | 1.36× (1.35–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.29) | 2.42× (2.39–2.45) | 2.17× (2.12–2.23) | 2.94× (2.91–2.97) | 2.51× (2.48–2.55) | 2.62× (2.59–2.65) | 2.72× (2.68–2.76) | 1.11× (1.09–1.12) | 1.22× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.43–1.49) | 2.28× (2.23–2.32) | 1.93× (1.90–1.95) | 2.64× (2.61–2.68) | 2.48× (2.44–2.51) | 1.99× (1.96–2.01) | 2.06× (2.04–2.09) | 0.84× (0.83–0.85) | 0.92× (0.91–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.52× (1.52–1.53) | 2.99× (2.99–3.00) | 2.63× (2.60–2.67) | 2.80× (2.78–2.82) | 3.12× (3.10–3.14) | 3.68× (3.67–3.69) | 3.96× (3.93–3.99) | 1.50× (1.49–1.50) | 1.74× (1.74–1.75) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.38–1.41) | 2.79× (2.75–2.83) | 2.37× (2.34–2.40) | 3.34× (3.30–3.38) | 2.78× (2.75–2.81) | 3.22× (3.19–3.26) | 3.47× (3.41–3.53) | 1.31× (1.30–1.33) | 1.53× (1.51–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.58–1.67) | 2.54× (2.48–2.59) | 2.03× (1.98–2.07) | 2.89× (2.84–2.95) | 2.64× (2.58–2.69) | 2.34× (2.29–2.38) | 2.51× (2.47–2.55) | 0.95× (0.93–0.97) | 1.10× (1.09–1.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.48× (2.45–2.51) | 3.41× (3.38–3.45) | 2.86× (2.82–2.89) | 5.99× (5.89–6.09) | 2.13× (2.11–2.15) | 2.16× (2.14–2.18) | 0.96× (0.94–0.98) | 1.37× (1.36–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.60× (0.59–0.61) | 2.29× (2.26–2.33) | 4.11× (4.03–4.18) | 2.79× (2.74–2.83) | 5.64× (5.53–5.76) | 2.03× (2.00–2.06) | 2.05× (2.02–2.09) | 0.91× (0.90–0.93) | 1.31× (1.29–1.33) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.53× (2.49–2.56) | 4.11× (4.05–4.16) | 2.83× (2.79–2.87) | 5.76× (5.64–5.87) | 2.05× (2.02–2.08) | 2.08× (2.05–2.11) | 0.92× (0.91–0.94) | 1.32× (1.31–1.34) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.33× (1.32–1.35) | 2.82× (2.78–2.86) | 3.09× (3.05–3.14) | 3.31× (3.27–3.36) | 3.47× (3.42–3.52) | 3.46× (3.41–3.50) | 3.41× (3.35–3.48) | 1.11× (1.09–1.13) | 1.21× (1.20–1.23) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.21× (1.20–1.22) | 2.54× (2.51–2.56) | 2.80× (2.77–2.82) | 3.31× (3.24–3.38) | 3.03× (3.00–3.06) | 2.96× (2.92–2.99) | 2.92× (2.87–2.96) | 0.95× (0.94–0.96) | 1.04× (1.02–1.05) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.39× (1.34–1.44) | 2.13× (2.06–2.21) | 2.44× (2.37–2.51) | 2.74× (2.65–2.84) | 2.64× (2.57–2.71) | 2.04× (1.99–2.10) | 2.02× (1.95–2.09) | 0.66× (0.64–0.67) | 0.72× (0.70–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.35× (0.34–0.37) | 1.07× (1.07–1.07) | 0.87× (0.86–0.87) | 12.09× (12.07–12.10) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 0.47× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.56× (0.48–0.66) | 0.79× (0.78–0.80) | 0.66× (0.65–0.67) | 6.33× (6.25–6.41) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.39–0.40) | 0.61× (0.50–0.74) | 0.70× (0.69–0.71) | 0.62× (0.61–0.64) | 4.13× (4.07–4.20) | 0.10× (0.10–0.10) | 0.10× (0.10–0.11) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.83–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.70–1.71) | 0.67× (0.66–0.67) | 8.05× (8.03–8.08) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.67–0.68) | 0.65× (0.61–0.70) | 1.63× (1.61–1.65) | 0.82× (0.80–0.83) | 5.91× (5.82–5.99) | 0.48× (0.47–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.67) | 0.70× (0.64–0.76) | 1.15× (1.09–1.20) | 0.70× (0.67–0.73) | 3.37× (3.23–3.52) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.74–0.74) | 5.22× (5.21–5.24) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 36 MB | 0.50× (0.48–0.52) | 0.41× (0.39–0.44) | 0.92× (0.89–0.96) | 0.74× (0.71–0.77) | 3.19× (3.06–3.33) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.64) | 0.45× (0.44–0.46) | 0.86× (0.84–0.87) | 0.81× (0.79–0.82) | 2.50× (2.45–2.54) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.23× (5.22–5.23) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.37× (0.35–0.40) | 0.84× (0.81–0.87) | 0.70× (0.67–0.74) | 2.89× (2.79–3.00) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.47–0.52) | 0.81× (0.79–0.82) | 0.80× (0.76–0.83) | 2.34× (2.30–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.25) | 1.08× (1.08–1.08) | 0.77× (0.77–0.78) | 5.40× (5.40–5.41) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.21) | 1.62× (1.61–1.63) | 0.77× (0.77–0.78) | 5.37× (5.35–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.78) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.40) | 1.47× (1.46–1.47) | 0.69× (0.69–0.69) | 6.63× (6.61–6.65) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 0.65× (0.65–0.65) | 0.32× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 38 MB | 0.45× (0.44–0.47) | 0.39× (0.36–0.42) | 1.05× (1.01–1.09) | 0.67× (0.64–0.70) | 3.33× (3.23–3.44) | 0.20× (0.20–0.21) | 0.21× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.51× (0.50–0.53) | 0.44× (0.40–0.47) | 0.93× (0.90–0.97) | 0.74× (0.71–0.77) | 2.39× (2.32–2.46) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 407 | 2.46 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 731 | 2.64 | 1.08× (1.06–1.10) | 88% | 1.02× (1.01–1.03) | 1.03× (1.02–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 855 | 4.37 | 1.79× (1.73–1.85) | 52% | 1.20× (1.18–1.21) | 1.67× (1.63–1.71) |

**Best:** mibz2 at 4 threads (855 MB/s); lbzip2 at 4 threads (247 MB/s); pbz2 at 4 threads (360 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.75, mibz2 4t 1.08, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.42, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 9%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.73 | 0.02 | 0.08 | 0.245 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.7% |
| MTF + RLE2 | 1.11 | 18.2% |
| T-vector build | 1.05 | 17.1% |
| walk (the one each build uses) | 2.86 | 46.8% |
| of which: segment assembly | 0.10 | 1.7% |
| RLE1 | 0.14 | 2.3% |
| CRC | 0.18 | 2.9% |
| **total** | 6.12 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.059 | (in its total) | 2.71× (2.70–2.71) |  |
| walk | 1.066 | 2.054 | (in its total) | 1.94× (1.92–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.041 | (in its total) | 0.95× (0.90–1.01) |  |
| inverse BWT, total | 1.502 | 3.154 | 1.889 | 2.11× (2.09–2.13) | 1.26× (1.25–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m52s |
| 4 threads | 36m09s ± 5m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 26m57s ± 3m58s |
| the scanner alone (its serial part) | 12m05s ± 30.2s |
| the workers alone, over 4 | 26m57s ± 3m58s |

At 4 threads the workers bound it. Per byte: the scanner 7.0 ns a byte of bz2, the workers 3.53 ns a byte of output at 4 threads (CPU, summed over them), 2.07 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
