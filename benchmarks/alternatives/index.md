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

The latest record: [`0e88842`](<https://github.com/lmmx/mibz2/commit/0e88842b5bd8b912fbd4e2a37dc583dce7a1447c>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42314183927>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.532 s | 394 | 6.6 | 2.27× (2.25–2.28) | 5.18× (5.15–5.20) | 5.37× (5.35–5.39) | 6.11× (6.09–6.13) | 5.36× (5.33–5.39) | 6.05× (6.02–6.07) | 5.30× (5.28–5.32) | 1.69× (1.68–1.70) | 1.88× (1.87–1.89) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.302 s | 694 | 7.1 | 2.38× (2.24–2.52) | 4.78× (4.53–5.03) | 5.57× (5.32–5.82) | 5.77× (5.52–6.03) | 5.04× (4.82–5.27) | 10.62× (10.16–11.10) | 9.31× (8.91–9.73) | 2.96× (2.83–3.10) | 3.30× (3.16–3.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.250 s | 840 | 11.6 | 2.29× (2.26–2.33) | 3.37× (3.33–3.41) | 4.18× (4.09–4.27) | 4.13× (4.08–4.18) | 4.06× (3.95–4.18) | 12.83× (12.60–13.06) | 11.25× (11.04–11.46) | 3.58× (3.50–3.65) | 3.99× (3.92–4.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,127 | 2.5 | 0.58× (0.57–0.59) | 3.45× (3.40–3.50) | 4.67× (4.53–4.81) | 3.60× (3.54–3.65) | 6.16× (6.06–6.26) | 2.72× (2.64–2.79) | 2.68× (2.64–2.72) | 0.97× (0.95–1.00) | 1.94× (1.75–2.15) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,102 | 2.6 | 0.52× (0.51–0.54) | 2.80× (2.73–2.88) | 5.66× (5.49–5.84) | 3.52× (3.43–3.61) | 5.93× (5.78–6.09) | 2.64× (2.58–2.70) | 2.60× (2.53–2.68) | 0.95× (0.91–0.99) | 1.88× (1.68–2.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,099 | 2.6 | 0.53× (0.53–0.54) | 2.82× (2.79–2.86) | 5.64× (5.57–5.71) | 3.53× (3.48–3.58) | 5.97× (5.84–6.11) | 2.65× (2.59–2.70) | 2.61× (2.58–2.65) | 0.95× (0.92–0.98) | 1.89× (1.69–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 684 | 4.0 | 2.48× (2.46–2.49) | 2.84× (2.83–2.86) | 3.56× (3.54–3.59) | 3.79× (3.77–3.81) | 4.13× (4.12–4.15) | 3.18× (3.15–3.21) | 2.10× (2.08–2.11) | 1.92× (1.91–1.93) | 2.84× (2.82–2.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,043 | 4.9 | 2.09× (1.98–2.21) | 2.73× (2.56–2.91) | 4.15× (3.94–4.37) | 3.44× (3.15–3.76) | 4.79× (4.59–4.99) | 4.75× (4.59–4.93) | 3.14× (3.02–3.26) | 2.87× (2.76–2.98) | 4.24× (4.09–4.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 897 | 9.3 | 1.70× (1.66–1.75) | 1.72× (1.60–1.86) | 2.89× (2.80–2.99) | 2.07× (1.99–2.15) | 3.78× (3.65–3.91) | 4.17× (4.05–4.30) | 2.76× (2.68–2.84) | 2.52× (2.45–2.59) | 3.72× (3.61–3.84) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.108 s | 74 | 35.6 | 1.39× (1.39–1.40) | 2.63× (2.61–2.66) | 2.44× (2.41–2.47) | 2.55× (2.53–2.57) | 2.85× (2.77–2.93) | 3.21× (3.19–3.23) | 3.03× (3.01–3.04) | 1.26× (1.25–1.27) | 1.40× (1.39–1.41) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.066 s | 122 | 40.2 | 1.31× (1.28–1.34) | 2.58× (2.46–2.70) | 2.39× (2.34–2.44) | 3.17× (3.09–3.25) | 2.94× (2.88–3.01) | 5.26× (5.17–5.36) | 4.97× (4.88–5.07) | 2.07× (2.02–2.11) | 2.30× (2.26–2.34) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 144 | 54.2 | 1.33× (1.31–1.36) | 2.14× (2.08–2.21) | 2.08× (2.02–2.14) | 2.65× (2.60–2.70) | 2.90× (2.85–2.95) | 6.26× (6.16–6.37) | 5.92× (5.83–6.00) | 2.46× (2.41–2.51) | 2.74× (2.69–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 32.3 | 1.50× (1.45–1.55) | 3.00× (2.89–3.11) | 2.66× (2.56–2.76) | 2.75× (2.68–2.82) | 3.05× (2.95–3.16) | 3.93× (3.81–4.06) | 3.83× (3.71–3.96) | 1.48× (1.43–1.53) | 1.78× (1.72–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 131 | 37.0 | 1.31× (1.24–1.38) | 2.84× (2.67–3.03) | 2.45× (2.33–2.59) | 3.39× (3.21–3.58) | 2.99× (2.84–3.14) | 6.26× (5.94–6.61) | 6.11× (5.78–6.45) | 2.35× (2.23–2.49) | 2.83× (2.68–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 155 | 52.1 | 1.43× (1.40–1.47) | 2.24× (2.18–2.31) | 2.25× (2.18–2.32) | 3.05× (2.94–3.16) | 2.87× (2.81–2.93) | 7.51× (7.36–7.67) | 7.33× (7.18–7.47) | 2.82× (2.77–2.88) | 3.39× (3.33–3.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 851 | 3.5 | 0.63× (0.62–0.63) | 3.49× (3.46–3.51) | 4.38× (4.35–4.42) | 3.13× (3.10–3.16) | 6.70× (6.59–6.82) | 2.35× (2.33–2.37) | 2.32× (2.30–2.34) | 0.97× (0.95–0.99) | 1.75× (1.74–1.77) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 810 | 3.7 | 0.52× (0.51–0.53) | 2.49× (2.44–2.53) | 4.99× (4.90–5.07) | 2.96× (2.90–3.01) | 6.31× (6.17–6.46) | 2.22× (2.18–2.26) | 2.19× (2.16–2.23) | 0.92× (0.90–0.93) | 1.66× (1.63–1.68) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 787 | 3.7 | 0.52× (0.51–0.53) | 2.45× (2.40–2.49) | 4.90× (4.82–4.98) | 2.93× (2.88–2.97) | 6.24× (6.11–6.37) | 2.18× (2.14–2.21) | 2.15× (2.12–2.19) | 0.90× (0.88–0.92) | 1.62× (1.60–1.65) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 0.065 s | 186 | 14.3 | 1.41× (1.40–1.42) | 2.98× (2.96–3.00) | 3.29× (3.27–3.32) | 3.49× (3.46–3.51) | 3.69× (3.67–3.72) | 3.80× (3.77–3.82) | 3.51× (3.47–3.56) | 1.15× (1.14–1.16) | 1.30× (1.25–1.35) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 0.041 s | 293 | 16.7 | 1.18× (1.14–1.23) | 2.59× (2.46–2.72) | 3.34× (3.19–3.49) | 3.58× (3.46–3.71) | 3.56× (3.44–3.69) | 5.88× (5.67–6.10) | 5.45× (5.26–5.63) | 1.79× (1.72–1.85) | 2.01× (1.89–2.14) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 0.036 s | 331 | 24.3 | 1.27× (1.21–1.34) | 2.07× (1.96–2.19) | 2.91× (2.83–2.99) | 3.01× (2.87–3.15) | 3.21× (3.13–3.28) | 6.75× (6.59–6.91) | 6.25× (6.11–6.39) | 2.05× (2.00–2.10) | 2.31× (2.19–2.44) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.26× (2.25–2.27) | 5.15× (5.13–5.18) | 5.35× (5.33–5.37) | 6.09× (6.07–6.11) | 5.34× (5.32–5.37) | 6.03× (6.00–6.05) | 5.29× (5.26–5.31) | 1.68× (1.67–1.69) | 1.87× (1.86–1.88) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.19× (2.16–2.21) | 4.90× (4.84–4.97) | 5.13× (5.07–5.18) | 5.99× (5.93–6.05) | 5.02× (4.96–5.07) | 5.61× (5.54–5.67) | 4.92× (4.86–4.97) | 1.57× (1.54–1.59) | 1.74× (1.72–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.09× (2.04–2.14) | 3.48× (3.44–3.52) | 3.73× (3.68–3.78) | 4.25× (4.19–4.30) | 3.65× (3.60–3.71) | 3.39× (3.34–3.45) | 2.98× (2.93–3.02) | 0.95× (0.93–0.96) | 1.05× (1.04–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.94× (2.90–2.98) | 3.95× (3.85–4.05) | 3.37× (3.32–3.42) | 5.71× (5.62–5.81) | 2.57× (2.50–2.65) | 2.54× (2.50–2.57) | 0.98× (0.96–1.00) | 1.66× (1.64–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.56–0.59) | 2.71× (2.64–2.78) | 4.92× (4.80–5.05) | 3.27× (3.20–3.35) | 5.43× (5.31–5.56) | 2.46× (2.41–2.52) | 2.43× (2.37–2.49) | 0.94× (0.90–0.97) | 1.59× (1.55–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.57–0.57) | 3.04× (3.00–3.08) | 4.98× (4.91–5.05) | 3.35× (3.31–3.40) | 5.57× (5.45–5.69) | 2.53× (2.47–2.58) | 2.49× (2.46–2.52) | 0.96× (0.93–0.99) | 1.63× (1.61–1.65) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.39× (2.38–2.41) | 2.64× (2.62–2.65) | 3.14× (3.11–3.16) | 3.64× (3.61–3.66) | 3.96× (3.94–3.99) | 3.06× (3.03–3.09) | 2.04× (2.02–2.05) | 1.87× (1.86–1.88) | 2.45× (2.43–2.48) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.10× (2.07–2.14) | 2.32× (2.26–2.37) | 3.12× (3.06–3.18) | 3.13× (3.08–3.17) | 3.40× (3.32–3.48) | 2.50× (2.46–2.54) | 1.66× (1.64–1.69) | 1.52× (1.50–1.55) | 2.00× (1.98–2.03) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.93× (1.84–2.03) | 1.64× (1.55–1.74) | 2.14× (2.02–2.26) | 2.01× (1.91–2.10) | 2.04× (1.95–2.14) | 1.30× (1.23–1.36) | 0.86× (0.82–0.91) | 0.79× (0.75–0.83) | 1.04× (0.99–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.39× (1.38–1.39) | 2.59× (2.58–2.61) | 2.38× (2.34–2.41) | 2.52× (2.51–2.54) | 2.82× (2.74–2.90) | 3.17× (3.15–3.19) | 3.00× (2.98–3.01) | 1.26× (1.24–1.27) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.30) | 2.45× (2.44–2.47) | 2.17× (2.16–2.18) | 2.97× (2.95–2.99) | 2.55× (2.53–2.57) | 2.83× (2.81–2.84) | 2.67× (2.66–2.68) | 1.12× (1.11–1.13) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.45–1.50) | 2.24× (2.19–2.31) | 1.91× (1.89–1.94) | 2.59× (2.55–2.63) | 2.47× (2.43–2.51) | 2.10× (2.06–2.13) | 1.98× (1.96–2.01) | 0.83× (0.81–0.85) | 0.91× (0.90–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.44–1.54) | 2.94× (2.85–3.04) | 2.58× (2.48–2.68) | 2.73× (2.66–2.80) | 3.02× (2.92–3.13) | 3.89× (3.77–4.02) | 3.79× (3.67–3.92) | 1.47× (1.42–1.52) | 1.71× (1.65–1.76) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.39) | 2.76× (2.74–2.78) | 2.35× (2.34–2.36) | 3.32× (3.29–3.34) | 2.77× (2.75–2.78) | 3.46× (3.44–3.49) | 3.38× (3.36–3.40) | 1.31× (1.30–1.32) | 1.52× (1.51–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.48–1.63) | 2.50× (2.44–2.57) | 2.00× (1.95–2.04) | 2.82× (2.74–2.91) | 2.61× (2.55–2.67) | 2.48× (2.42–2.53) | 2.41× (2.36–2.47) | 0.94× (0.91–0.96) | 1.09× (1.06–1.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.46× (2.44–2.47) | 3.40× (3.38–3.42) | 2.86× (2.83–2.88) | 5.98× (5.88–6.08) | 2.17× (2.16–2.19) | 2.15× (2.14–2.17) | 0.97× (0.95–0.99) | 1.37× (1.36–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.25× (2.22–2.27) | 3.98× (3.93–4.04) | 2.73× (2.69–2.77) | 5.59× (5.47–5.72) | 2.03× (2.00–2.06) | 2.01× (1.99–2.04) | 0.91× (0.90–0.92) | 1.28× (1.26–1.31) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.57) | 2.46× (2.43–2.50) | 3.96× (3.88–4.04) | 2.78× (2.73–2.82) | 5.68× (5.57–5.81) | 2.05× (2.02–2.08) | 2.03× (2.00–2.07) | 0.92× (0.89–0.94) | 1.29× (1.27–1.32) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 1.40× (1.39–1.41) | 2.88× (2.86–2.90) | 3.19× (3.16–3.21) | 3.42× (3.40–3.45) | 3.62× (3.60–3.65) | 3.72× (3.69–3.75) | 3.45× (3.40–3.49) | 1.15× (1.14–1.16) | 1.25× (1.24–1.26) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 1.26× (1.22–1.29) | 2.54× (2.47–2.61) | 2.84× (2.75–2.94) | 3.27× (3.16–3.39) | 3.14× (3.03–3.25) | 3.15× (3.04–3.27) | 2.92× (2.81–3.04) | 0.97× (0.94–1.01) | 1.06× (1.02–1.10) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 1.39× (1.33–1.46) | 2.18× (2.12–2.23) | 2.43× (2.38–2.48) | 2.69× (2.64–2.75) | 2.65× (2.60–2.70) | 2.19× (2.15–2.22) | 2.03× (1.98–2.08) | 0.67× (0.66–0.69) | 0.73× (0.72–0.75) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.36× (0.35–0.37) | 1.08× (1.08–1.09) | 0.86× (0.86–0.87) | 12.24× (12.23–12.26) | 0.29× (0.28–0.29) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.53× (0.45–0.63) | 0.81× (0.79–0.82) | 0.66× (0.66–0.67) | 6.39× (6.35–6.44) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.53× (0.45–0.63) | 0.70× (0.69–0.71) | 0.61× (0.61–0.62) | 4.10× (4.08–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.44× (4.43–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.83–0.84) | 4.44× (4.43–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.46) | 1.74× (1.74–1.75) | 0.67× (0.66–0.67) | 8.21× (8.19–8.23) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.38× (0.38–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.69) | 0.65× (0.58–0.72) | 1.63× (1.58–1.67) | 0.79× (0.77–0.82) | 5.88× (5.72–6.03) | 0.47× (0.45–0.48) | 0.49× (0.47–0.51) | 0.54× (0.52–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 41 MB | 0.60× (0.57–0.64) | 0.68× (0.64–0.73) | 1.11× (1.05–1.18) | 0.72× (0.68–0.77) | 3.25× (3.09–3.42) | 0.25× (0.24–0.26) | 0.26× (0.25–0.28) | 0.29× (0.27–0.30) | 0.14× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.19× (1.19–1.20) | 0.74× (0.74–0.74) | 5.28× (5.27–5.29) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.42× (0.40–0.45) | 0.93× (0.90–0.96) | 0.71× (0.68–0.73) | 3.21× (3.09–3.33) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.83× (0.81–0.84) | 2.55× (2.51–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.29× (5.28–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.47) | 0.39× (0.38–0.40) | 0.85× (0.83–0.88) | 0.71× (0.69–0.74) | 2.91× (2.83–3.00) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.52) | 0.81× (0.80–0.82) | 0.82× (0.79–0.85) | 2.33× (2.30–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.24× (0.22–0.25) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.47× (5.47–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.63–1.65) | 0.77× (0.76–0.77) | 5.42× (5.40–5.45) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.77) | 5.44× (5.43–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.50× (1.50–1.51) | 0.65× (0.65–0.65) | 6.78× (6.76–6.80) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 0.66× (0.66–0.67) | 0.33× (0.33–0.33) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 32 MB | 0.50× (0.49–0.50) | 0.42× (0.38–0.46) | 1.21× (1.18–1.23) | 0.75× (0.74–0.76) | 3.82× (3.77–3.86) | 0.23× (0.23–0.23) | 0.24× (0.24–0.24) | 0.37× (0.37–0.38) | 0.19× (0.18–0.19) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 48 MB | 0.56× (0.54–0.58) | 0.56× (0.52–0.60) | 1.06× (1.04–1.09) | 0.84× (0.80–0.87) | 2.71× (2.65–2.78) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.26) | 0.13× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 394 | 2.55 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 694 | 2.72 | 1.07× (1.06–1.09) | 88% | 1.02× (1.02–1.03) | 1.04× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 840 | 4.48 | 1.78× (1.75–1.80) | 53% | 1.20× (1.19–1.21) | 1.64× (1.60–1.68) |

**Best:** mibz2 at 4 threads (840 MB/s); lbzip2 at 4 threads (249 MB/s); pbz2 at 4 threads (367 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.10, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.32, pbz2 1t 0.00, pbz2 2t 0.43, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.75 | 0.02 | 0.09 | 0.250 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 52.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.5% |
| MTF + RLE2 | 1.06 | 17.2% |
| T-vector build | 1.24 | 20.0% |
| walk (the one each build uses) | 2.80 | 45.1% |
| of which: segment assembly | 0.10 | 1.6% |
| RLE1 | 0.14 | 2.3% |
| CRC | 0.18 | 2.9% |
| **total** | 6.20 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.452 | 1.061 | (in its total) | 2.35× (2.34–2.36) |  |
| walk | 1.028 | 2.033 | (in its total) | 1.98× (1.97–1.98) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.040 | (in its total) | 1.10× (1.09–1.10) |  |
| inverse BWT, total | 1.517 | 3.134 | 1.872 | 2.07× (2.06–2.07) | 1.23× (1.23–1.24) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m12s |
| 4 threads | 36m32s ± 5m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m37s ± 3m55s |
| the scanner alone (its serial part) | 11m56s ± 26.3s |
| the workers alone, over 4 | 27m36s ± 3m55s |

At 4 threads the workers bound it. Per byte: the scanner 6.9 ns a byte of bz2, the workers 3.62 ns a byte of output at 4 threads (CPU, summed over them), 2.15 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,758,792 | 12,027,975 | \`3c64b8b3acbf9c14…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
