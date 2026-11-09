# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-311/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`97eb8ed`](<https://github.com/lmmx/mibz2/commit/97eb8edf51c558a7f74da595c152ab75288f531e>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42690920630>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.4.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.699 s | 300 | 9.6 | 2.32× (2.29–2.36) | 5.33× (5.26–5.40) | 5.46× (5.36–5.56) | 6.18× (6.10–6.27) | 5.52× (5.39–5.65) | 6.17× (6.07–6.27) | 5.46× (5.37–5.54) | 1.72× (1.69–1.75) | 1.88× (1.84–1.92) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.393 s | 534 | 10.4 | 2.32× (2.27–2.38) | 4.91× (4.76–5.07) | 5.59× (5.41–5.77) | 5.90× (5.71–6.10) | 5.22× (5.08–5.37) | 10.99× (10.65–11.34) | 9.72× (9.51–9.93) | 3.06× (3.00–3.12) | 3.35× (3.22–3.47) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.332 s | 632 | 17.3 | 2.37× (2.31–2.43) | 3.57× (3.46–3.69) | 4.33× (4.20–4.46) | 4.37× (4.24–4.49) | 4.22× (4.13–4.31) | 13.14× (12.94–13.34) | 11.62× (11.48–11.76) | 3.66× (3.58–3.74) | 4.00× (3.88–4.13) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 891 | 3.5 | 0.59× (0.58–0.59) | 3.45× (3.29–3.62) | 4.43× (4.37–4.48) | 3.63× (3.59–3.68) | 6.17× (6.10–6.25) | 2.74× (2.71–2.77) | 2.73× (2.69–2.77) | 0.99× (0.98–1.01) | 1.82× (1.80–1.84) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 s | 881 | 3.6 | 0.55× (0.54–0.56) | 2.76× (2.65–2.87) | 5.75× (5.67–5.84) | 3.61× (3.55–3.67) | 5.93× (5.83–6.04) | 2.71× (2.67–2.75) | 2.70× (2.66–2.74) | 0.98× (0.96–1.01) | 1.80× (1.78–1.83) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 856 | 3.6 | 0.54× (0.53–0.55) | 2.79× (2.67–2.91) | 5.66× (5.55–5.77) | 3.55× (3.47–3.63) | 6.01× (5.87–6.14) | 2.66× (2.61–2.71) | 2.65× (2.58–2.72) | 0.97× (0.94–0.99) | 1.77× (1.74–1.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.037 s | 528 | 5.8 | 2.47× (2.44–2.50) | 2.75× (2.72–2.78) | 3.45× (3.37–3.53) | 3.80× (3.72–3.87) | 4.09× (4.03–4.15) | 3.17× (3.13–3.20) | 2.09× (2.07–2.11) | 1.96× (1.93–1.99) | 2.76× (2.72–2.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 825 | 7.0 | 2.09× (1.90–2.31) | 2.73× (2.30–3.24) | 3.84× (3.50–4.20) | 3.35× (3.02–3.71) | 4.63× (4.23–5.06) | 4.72× (4.33–5.14) | 3.11× (2.85–3.39) | 2.91× (2.66–3.19) | 4.12× (3.76–4.51) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 751 | 13.4 | 1.85× (1.79–1.92) | 1.76× (1.62–1.91) | 2.84× (2.68–3.00) | 2.11× (2.02–2.20) | 3.95× (3.79–4.12) | 4.45× (4.30–4.61) | 2.94× (2.84–3.04) | 2.75× (2.65–2.86) | 3.89× (3.72–4.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.144 s | 56 | 52.5 | 1.37× (1.31–1.42) | 2.56× (2.46–2.66) | 2.28× (2.20–2.36) | 2.37× (2.28–2.46) | 2.68× (2.58–2.78) | 3.09× (2.98–3.21) | 2.94× (2.83–3.04) | 1.25× (1.20–1.31) | 1.40× (1.36–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.088 s | 91 | 58.9 | 1.29× (1.27–1.31) | 2.43× (2.37–2.49) | 2.29× (2.21–2.38) | 3.04× (2.93–3.15) | 2.81× (2.74–2.89) | 5.10× (5.00–5.21) | 4.84× (4.74–4.95) | 2.07× (2.00–2.14) | 2.31× (2.25–2.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 112 | 76.5 | 1.34× (1.31–1.38) | 2.13× (2.07–2.19) | 2.14× (2.06–2.23) | 2.67× (2.59–2.76) | 2.92× (2.81–3.04) | 6.27× (6.15–6.40) | 5.96× (5.83–6.09) | 2.54× (2.46–2.63) | 2.83× (2.76–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.154 s | 62 | 46.9 | 1.51× (1.44–1.57) | 2.96× (2.84–3.09) | 2.54× (2.44–2.65) | 2.74× (2.61–2.88) | 3.11× (2.92–3.31) | 3.90× (3.74–4.07) | 3.81× (3.65–3.96) | 1.52× (1.46–1.59) | 1.77× (1.69–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.093 s | 103 | 53.0 | 1.39× (1.37–1.42) | 2.79× (2.74–2.85) | 2.54× (2.47–2.60) | 3.64× (3.54–3.74) | 3.14× (3.09–3.19) | 6.60× (6.50–6.69) | 6.43× (6.34–6.52) | 2.57× (2.53–2.62) | 2.99× (2.95–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 121 | 73.7 | 1.46× (1.42–1.49) | 2.34× (2.27–2.41) | 2.28× (2.21–2.37) | 3.21× (3.13–3.30) | 2.94× (2.85–3.03) | 7.70× (7.58–7.83) | 7.51× (7.37–7.66) | 3.01× (2.92–3.10) | 3.49× (3.42–3.56) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 673 | 4.9 | 0.64× (0.63–0.66) | 2.90× (2.71–3.09) | 3.98× (3.93–4.04) | 3.16× (3.12–3.20) | 6.90× (6.76–7.03) | 2.40× (2.37–2.44) | 2.38× (2.35–2.41) | 0.96× (0.95–0.97) | 2.07× (2.05–2.10) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 s | 629 | 5.3 | 0.52× (0.51–0.54) | 2.60× (2.54–2.66) | 4.96× (4.78–5.16) | 2.98× (2.92–3.05) | 6.34× (6.21–6.46) | 2.25× (2.20–2.30) | 2.23× (2.18–2.28) | 0.90× (0.87–0.92) | 1.94× (1.90–1.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 627 | 5.1 | 0.54× (0.52–0.55) | 2.61× (2.57–2.64) | 4.91× (4.71–5.11) | 3.00× (2.95–3.04) | 6.39× (6.25–6.53) | 2.25× (2.21–2.29) | 2.23× (2.19–2.26) | 0.90× (0.88–0.92) | 1.94× (1.91–1.97) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 0.095 s | 130 | 22.7 | 1.34× (1.32–1.35) | 2.88× (2.81–2.95) | 3.11× (3.08–3.15) | 3.35× (3.31–3.38) | 3.50× (3.46–3.53) | 3.64× (3.58–3.71) | 3.29× (3.23–3.36) | 1.09× (1.07–1.10) | 1.25× (1.22–1.28) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 0.060 s | 203 | 26.5 | 1.13× (1.11–1.16) | 2.56× (2.42–2.71) | 2.97× (2.87–3.09) | 3.52× (3.44–3.61) | 3.40× (3.32–3.48) | 5.72× (5.58–5.87) | 5.17× (5.05–5.30) | 1.71× (1.67–1.75) | 1.96× (1.89–2.03) | yes |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 0.049 s | 250 | 38.0 | 1.27× (1.22–1.32) | 2.13× (1.98–2.30) | 2.69× (2.50–2.90) | 3.17× (3.10–3.24) | 3.53× (3.39–3.69) | 7.08× (6.89–7.27) | 6.40× (6.23–6.57) | 2.11× (2.07–2.16) | 2.42× (2.36–2.49) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.32× (2.29–2.35) | 5.31× (5.24–5.38) | 5.44× (5.34–5.53) | 6.17× (6.09–6.25) | 5.51× (5.38–5.63) | 6.15× (6.05–6.25) | 5.44× (5.36–5.53) | 1.72× (1.69–1.75) | 1.87× (1.83–1.91) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.22× (2.19–2.26) | 4.99× (4.90–5.08) | 5.26× (5.17–5.35) | 6.02× (5.94–6.11) | 5.09× (5.03–5.15) | 5.67× (5.59–5.76) | 5.02× (4.96–5.07) | 1.58× (1.55–1.61) | 1.72× (1.68–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.20× (2.15–2.25) | 3.66× (3.54–3.79) | 3.96× (3.85–4.07) | 4.44× (4.33–4.54) | 3.84× (3.75–3.93) | 3.45× (3.39–3.51) | 3.05× (3.01–3.09) | 0.96× (0.94–0.98) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.62–0.63) | 3.00× (2.97–3.03) | 3.98× (3.94–4.02) | 3.40× (3.37–3.44) | 5.73× (5.66–5.79) | 2.59× (2.56–2.61) | 2.58× (2.55–2.61) | 0.99× (0.98–1.01) | 1.67× (1.65–1.68) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.60× (0.59–0.61) | 2.82× (2.78–2.86) | 5.04× (4.96–5.12) | 3.36× (3.32–3.41) | 5.45× (5.36–5.54) | 2.53× (2.49–2.56) | 2.52× (2.49–2.55) | 0.97× (0.95–0.99) | 1.63× (1.60–1.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.57–0.59) | 3.07× (3.01–3.14) | 5.00× (4.91–5.09) | 3.38× (3.31–3.45) | 5.61× (5.49–5.73) | 2.53× (2.49–2.58) | 2.53× (2.47–2.59) | 0.97× (0.95–0.99) | 1.63× (1.60–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.39× (2.37–2.42) | 2.63× (2.60–2.66) | 3.14× (3.12–3.16) | 3.65× (3.58–3.72) | 3.93× (3.88–3.98) | 3.05× (3.02–3.09) | 2.03× (2.01–2.05) | 1.91× (1.88–1.93) | 2.46× (2.41–2.51) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.14× (2.11–2.17) | 2.37× (2.31–2.43) | 3.12× (3.09–3.16) | 3.14× (3.10–3.17) | 3.35× (3.29–3.40) | 2.51× (2.48–2.54) | 1.67× (1.65–1.69) | 1.56× (1.54–1.59) | 2.02× (1.97–2.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.95–2.03) | 1.60× (1.54–1.67) | 2.16× (2.10–2.21) | 2.02× (1.96–2.09) | 2.03× (1.98–2.08) | 1.31× (1.28–1.34) | 0.87× (0.85–0.89) | 0.82× (0.80–0.84) | 1.05× (1.02–1.09) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.31–1.41) | 2.52× (2.42–2.61) | 2.22× (2.14–2.30) | 2.35× (2.27–2.44) | 2.66× (2.56–2.76) | 3.06× (2.95–3.17) | 2.91× (2.81–3.01) | 1.25× (1.20–1.30) | 1.35× (1.31–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.29) | 2.42× (2.38–2.45) | 2.10× (2.08–2.11) | 2.87× (2.85–2.90) | 2.48× (2.45–2.51) | 2.77× (2.75–2.79) | 2.63× (2.61–2.65) | 1.13× (1.10–1.16) | 1.22× (1.20–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.51× (1.48–1.54) | 2.35× (2.28–2.41) | 1.95× (1.89–2.02) | 2.71× (2.63–2.79) | 2.58× (2.48–2.67) | 2.14× (2.11–2.17) | 2.03× (2.00–2.06) | 0.87× (0.85–0.89) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.44–1.56) | 2.91× (2.79–3.03) | 2.49× (2.40–2.59) | 2.72× (2.59–2.85) | 3.08× (2.90–3.28) | 3.86× (3.70–4.03) | 3.77× (3.62–3.92) | 1.52× (1.45–1.58) | 1.72× (1.65–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.38–1.43) | 2.77× (2.74–2.80) | 2.34× (2.30–2.39) | 3.33× (3.29–3.37) | 2.79× (2.77–2.82) | 3.47× (3.44–3.51) | 3.39× (3.36–3.42) | 1.36× (1.34–1.39) | 1.54× (1.52–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.59–1.65) | 2.61× (2.52–2.69) | 2.03× (1.95–2.11) | 3.00× (2.88–3.13) | 2.71× (2.60–2.82) | 2.51× (2.46–2.56) | 2.45× (2.41–2.49) | 0.98× (0.96–1.01) | 1.11× (1.09–1.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.53× (2.50–2.56) | 3.50× (3.47–3.53) | 2.90× (2.87–2.93) | 6.19× (6.08–6.30) | 2.24× (2.21–2.26) | 2.22× (2.20–2.24) | 0.97× (0.96–0.98) | 1.40× (1.39–1.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.29× (2.24–2.34) | 4.06× (3.97–4.16) | 2.76× (2.71–2.82) | 5.62× (5.53–5.71) | 2.06× (2.02–2.10) | 2.04× (2.00–2.08) | 0.89× (0.87–0.92) | 1.29× (1.26–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.58–0.60) | 2.56× (2.53–2.60) | 4.12× (4.05–4.18) | 2.86× (2.82–2.90) | 5.85× (5.73–5.97) | 2.12× (2.09–2.16) | 2.11× (2.08–2.13) | 0.92× (0.90–0.94) | 1.33× (1.31–1.35) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 1.33× (1.32–1.35) | 2.81× (2.73–2.89) | 3.04× (3.01–3.07) | 3.30× (3.26–3.33) | 3.44× (3.41–3.48) | 3.58× (3.52–3.65) | 3.24× (3.18–3.30) | 1.09× (1.07–1.10) | 1.18× (1.17–1.20) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 1.20× (1.19–1.22) | 2.53× (2.47–2.58) | 2.82× (2.75–2.89) | 3.25× (3.21–3.28) | 3.03× (2.98–3.08) | 3.07× (3.01–3.13) | 2.78× (2.72–2.84) | 0.93× (0.92–0.94) | 1.01× (0.99–1.03) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 1.42× (1.37–1.46) | 2.30× (2.15–2.46) | 2.57× (2.44–2.72) | 2.81× (2.74–2.88) | 2.84× (2.69–3.01) | 2.16× (2.13–2.19) | 1.96× (1.93–1.98) | 0.65× (0.65–0.66) | 0.71× (0.70–0.72) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.32× (0.31–0.34) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.11× (12.10–12.12) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.47× (0.40–0.55) | 0.80× (0.79–0.81) | 0.67× (0.66–0.68) | 6.33× (6.26–6.39) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.39) | 0.59× (0.50–0.69) | 0.70× (0.69–0.71) | 0.62× (0.60–0.63) | 4.09× (4.07–4.11) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.16× (0.16–0.16) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.30–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.15–0.18) | 1.79× (1.78–1.79) | 0.84× (0.83–0.84) | 4.41× (4.41–4.42) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.15–0.17) | 1.78× (1.78–1.78) | 0.84× (0.83–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.45) | 1.71× (1.71–1.71) | 0.66× (0.66–0.67) | 8.07× (8.06–8.08) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.67–0.69) | 0.65× (0.60–0.71) | 1.65× (1.63–1.68) | 0.82× (0.81–0.83) | 5.97× (5.88–6.06) | 0.48× (0.47–0.49) | 0.51× (0.50–0.51) | 0.54× (0.54–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.60–0.67) | 0.65× (0.60–0.71) | 1.18× (1.12–1.23) | 0.71× (0.68–0.75) | 3.43× (3.28–3.58) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.30× (0.29–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.22–5.23) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.39× (0.35–0.43) | 0.92× (0.88–0.96) | 0.73× (0.69–0.77) | 3.18× (3.05–3.31) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.63× (0.61–0.65) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.80× (0.78–0.83) | 2.54× (2.49–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.84× (0.83–0.84) | 5.24× (5.23–5.25) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.51–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.38× (0.35–0.40) | 0.86× (0.83–0.88) | 0.72× (0.69–0.74) | 2.94× (2.85–3.02) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.59–0.63) | 0.47× (0.44–0.51) | 0.81× (0.80–0.83) | 0.82× (0.79–0.85) | 2.34× (2.31–2.38) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.09) | 0.77× (0.77–0.78) | 5.42× (5.41–5.43) | 0.35× (0.35–0.36) | 0.38× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.21× (0.20–0.21) | 1.62× (1.61–1.63) | 0.77× (0.76–0.77) | 5.35× (5.33–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.21) | 1.63× (1.63–1.63) | 0.77× (0.77–0.78) | 5.39× (5.37–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.41) | 1.48× (1.47–1.48) | 0.68× (0.68–0.69) | 6.65× (6.64–6.66) | 0.40× (0.40–0.41) | 0.44× (0.43–0.44) | 0.65× (0.65–0.65) | 0.32× (0.32–0.32) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 2 | 34 MB | 0.48× (0.47–0.49) | 0.42× (0.39–0.45) | 1.15× (1.12–1.17) | 0.74× (0.72–0.75) | 3.62× (3.56–3.68) | 0.22× (0.22–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.18× (0.17–0.18) |
| `text` (2.26 MB → 12.3 MB, 5.4×) | 4 | 52 MB | 0.53× (0.52–0.54) | 0.46× (0.42–0.49) | 0.97× (0.95–0.99) | 0.79× (0.76–0.81) | 2.48× (2.43–2.52) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 300 | 3.34 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 534 | 3.63 | 1.08× (1.07–1.10) | 89% | 1.02× (1.00–1.04) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 632 | 6.02 | 1.78× (1.75–1.81) | 53% | 1.23× (1.19–1.27) | 1.69× (1.65–1.73) |

**Best:** mibz2 at 4 threads (632 MB/s); lbzip2 at 4 threads (180 MB/s); pbz2 at 4 threads (269 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 1.04, lbzip2 1t 0.07, lbzip2 2t 0.21, lbzip2 4t 0.38, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.04 | 0.02 | 0.09 | 0.332 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2876 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 12.3% |
| MTF + RLE2 | 1.53 | 16.8% |
| T-vector build | 1.79 | 19.7% |
| walk (the one each build uses) | 4.19 | 46.1% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.20 | 2.2% |
| CRC | 0.26 | 2.8% |
| **total** | 9.09 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.584 | 1.364 | (in its total) | 2.34× (2.34–2.34) |  |
| walk | 1.341 | 2.622 | (in its total) | 1.95× (1.94–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.051 | (in its total) | 1.07× (1.03–1.11) |  |
| inverse BWT, total | 1.972 | 4.039 | 2.461 | 2.04× (2.03–2.06) | 1.25× (1.24–1.25) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h42m ± 12m14s |
| 4 threads | 48m01s ± 6m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 37m48s ± 5m29s |
| the scanner alone (its serial part) | 16m18s ± 26.3s |
| the workers alone, over 4 | 37m48s ± 5m29s |

At 4 threads the workers bound it. Per byte: the scanner 9.4 ns a byte of bz2, the workers 4.96 ns a byte of output at 4 threads (CPU, summed over them), 2.82 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,257,451 | 12,280,075 | \`b7e3edb779637a6e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2875.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.4.0 (bench/report.py), 10 rounds, threads 1, 2, 4
