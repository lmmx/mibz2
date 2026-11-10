# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-357/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`8ed2fbd`](<https://github.com/lmmx/mibz2/commit/8ed2fbdf3e8c05cdbfa426676883c652054ef17d>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42819287351>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.532 s | 394 | 6.2 | 2.88× (2.85–2.90) | 6.62× (6.57–6.66) | 6.84× (6.77–6.91) | 7.43× (7.36–7.50) | 6.72× (6.69–6.76) | 7.29× (7.25–7.33) | 6.84× (6.80–6.88) | 2.12× (2.10–2.14) | 2.47× (2.45–2.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.330 s | 636 | 7.2 | 2.64× (2.46–2.84) | 5.86× (5.52–6.23) | 6.68× (6.25–7.14) | 6.50× (6.09–6.93) | 5.95× (5.62–6.30) | 11.62× (11.02–12.25) | 10.91× (10.33–11.51) | 3.38× (3.19–3.58) | 3.94× (3.73–4.18) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.282 s | 744 | 12.2 | 2.57× (2.52–2.61) | 3.98× (3.90–4.07) | 4.84× (4.72–4.97) | 4.42× (4.32–4.52) | 4.56× (4.44–4.69) | 13.70× (13.46–13.93) | 12.86× (12.63–13.09) | 3.98× (3.91–4.06) | 4.65× (4.56–4.74) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 948 | 2.8 | 0.61× (0.60–0.62) | 3.89× (3.81–3.97) | 4.72× (4.59–4.84) | 3.52× (3.44–3.59) | 6.09× (5.97–6.22) | 2.94× (2.88–3.01) | 2.62× (2.57–2.67) | 1.02× (1.00–1.03) | 1.47× (1.44–1.50) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 s | 926 | 3.0 | 0.60× (0.59–0.61) | 2.86× (2.81–2.90) | 6.08× (5.98–6.17) | 3.45× (3.38–3.51) | 5.89× (5.81–5.98) | 2.87× (2.82–2.91) | 2.55× (2.51–2.59) | 0.99× (0.96–1.02) | 1.43× (1.41–1.45) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 930 | 2.9 | 0.60× (0.58–0.61) | 2.88× (2.84–2.93) | 6.06× (5.93–6.19) | 3.44× (3.39–3.50) | 5.98× (5.89–6.08) | 2.88× (2.83–2.92) | 2.56× (2.52–2.60) | 0.99× (0.96–1.03) | 1.43× (1.41–1.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 559 | 4.6 | 2.60× (2.57–2.64) | 2.92× (2.90–2.94) | 3.81× (3.69–3.94) | 3.68× (3.66–3.71) | 4.25× (4.20–4.29) | 2.50× (2.48–2.52) | 2.11× (2.09–2.13) | 2.00× (1.96–2.03) | 2.90× (2.88–2.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 843 | 5.9 | 2.10× (1.90–2.32) | 2.74× (2.48–3.02) | 3.90× (3.46–4.38) | 3.17× (2.83–3.54) | 4.68× (4.23–5.17) | 3.59× (3.25–3.97) | 3.03× (2.74–3.36) | 2.87× (2.59–3.18) | 4.17× (3.77–4.61) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 730 | 11.4 | 1.74× (1.69–1.79) | 1.91× (1.83–2.00) | 2.83× (2.68–2.99) | 2.11× (2.02–2.19) | 3.71× (3.58–3.85) | 3.19× (3.08–3.30) | 2.69× (2.60–2.78) | 2.55× (2.47–2.64) | 3.70× (3.58–3.83) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 s | 60 | 41.1 | 1.34× (1.32–1.37) | 2.73× (2.67–2.78) | 2.45× (2.37–2.53) | 2.47× (2.41–2.53) | 2.89× (2.82–2.96) | 3.16× (3.10–3.23) | 3.22× (3.15–3.28) | 1.23× (1.20–1.26) | 1.41× (1.38–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 97 | 46.6 | 1.28× (1.26–1.30) | 2.58× (2.51–2.66) | 2.44× (2.41–2.47) | 3.12× (3.01–3.24) | 2.96× (2.94–2.98) | 5.12× (5.06–5.17) | 5.20× (5.15–5.26) | 1.99× (1.97–2.01) | 2.29× (2.25–2.32) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 64.5 | 1.34× (1.30–1.38) | 2.14× (2.08–2.20) | 2.08× (1.98–2.17) | 2.58× (2.53–2.63) | 2.84× (2.76–2.93) | 6.03× (5.89–6.16) | 6.13× (5.98–6.28) | 2.34× (2.28–2.41) | 2.69× (2.60–2.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 38.4 | 1.45× (1.43–1.47) | 3.07× (3.00–3.14) | 2.55× (2.51–2.60) | 2.62× (2.58–2.66) | 3.10× (3.04–3.17) | 3.62× (3.57–3.67) | 3.98× (3.92–4.04) | 1.41× (1.38–1.44) | 1.70× (1.67–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 105 | 44.4 | 1.33× (1.30–1.36) | 2.73× (2.68–2.79) | 2.47× (2.42–2.52) | 3.45× (3.32–3.59) | 3.06× (2.99–3.13) | 5.87× (5.77–5.96) | 6.45× (6.34–6.56) | 2.29× (2.24–2.33) | 2.76× (2.69–2.82) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.077 s | 124 | 61.2 | 1.45× (1.40–1.49) | 2.35× (2.26–2.44) | 2.26× (2.18–2.34) | 2.98× (2.88–3.09) | 2.86× (2.82–2.91) | 6.92× (6.82–7.03) | 7.61× (7.49–7.74) | 2.70× (2.64–2.76) | 3.25× (3.16–3.34) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 709 | 4.0 | 0.66× (0.64–0.68) | 3.38× (3.10–3.69) | 4.20× (4.08–4.33) | 3.03× (2.93–3.12) | 6.68× (6.44–6.93) | 2.49× (2.42–2.57) | 2.30× (2.24–2.37) | 0.96× (0.94–0.99) | 1.46× (1.41–1.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 4.3 | 0.60× (0.58–0.61) | 2.74× (2.69–2.80) | 5.19× (4.96–5.43) | 2.92× (2.85–2.98) | 6.31× (6.19–6.44) | 2.36× (2.31–2.41) | 2.18× (2.12–2.23) | 0.91× (0.89–0.93) | 1.38× (1.35–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 652 | 4.2 | 0.57× (0.56–0.59) | 2.71× (2.67–2.75) | 5.11× (4.94–5.28) | 2.86× (2.82–2.90) | 6.20× (6.10–6.30) | 2.32× (2.28–2.35) | 2.14× (2.10–2.17) | 0.89× (0.88–0.91) | 1.35× (1.33–1.37) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 0.074 s | 174 | 14.5 | 1.93× (1.90–1.97) | 4.05× (3.96–4.14) | 4.42× (4.34–4.51) | 4.41× (4.30–4.51) | 4.66× (4.54–4.78) | 5.10× (4.99–5.22) | 4.88× (4.76–5.00) | 1.50× (1.46–1.55) | 1.85× (1.79–1.91) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 0.054 s | 238 | 18.6 | 1.51× (1.45–1.58) | 3.04× (2.84–3.27) | 3.61× (3.34–3.90) | 3.82× (3.56–4.10) | 3.89× (3.64–4.15) | 6.71× (6.29–7.15) | 6.42× (6.04–6.83) | 1.98× (1.85–2.11) | 2.43× (2.27–2.60) | yes |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 0.046 s | 280 | 28.4 | 1.57× (1.52–1.61) | 2.44× (2.36–2.52) | 3.22× (3.11–3.33) | 3.33× (3.17–3.49) | 3.76× (3.64–3.88) | 8.08× (7.96–8.20) | 7.73× (7.64–7.82) | 2.38× (2.31–2.46) | 2.93× (2.84–3.01) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.87× (2.84–2.90) | 6.59× (6.54–6.64) | 6.81× (6.74–6.88) | 7.41× (7.34–7.48) | 6.70× (6.66–6.74) | 7.26× (7.22–7.30) | 6.82× (6.78–6.86) | 2.12× (2.09–2.14) | 2.46× (2.44–2.48) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.63× (2.58–2.67) | 6.04× (5.95–6.13) | 6.42× (6.27–6.57) | 6.93× (6.79–7.09) | 6.07× (5.93–6.22) | 6.28× (6.19–6.38) | 5.90× (5.81–5.99) | 1.83× (1.80–1.86) | 2.13× (2.10–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.49× (2.45–2.54) | 4.21× (4.12–4.30) | 4.49× (4.39–4.59) | 4.71× (4.63–4.80) | 4.28× (4.18–4.38) | 3.76× (3.71–3.81) | 3.53× (3.47–3.60) | 1.10× (1.08–1.11) | 1.27× (1.25–1.29) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.66) | 3.25× (3.19–3.31) | 4.01× (3.94–4.07) | 3.29× (3.23–3.35) | 5.63× (5.52–5.73) | 2.76× (2.71–2.82) | 2.47× (2.43–2.52) | 1.01× (1.00–1.03) | 1.07× (1.03–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.64× (0.63–0.65) | 3.03× (2.98–3.08) | 5.28× (5.19–5.37) | 3.19× (3.14–3.25) | 5.37× (5.30–5.43) | 2.65× (2.61–2.69) | 2.37× (2.33–2.40) | 0.97× (0.95–1.00) | 1.02× (0.99–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.63× (0.62–0.65) | 3.42× (3.35–3.49) | 5.37× (5.30–5.43) | 3.26× (3.21–3.32) | 5.53× (5.45–5.62) | 2.71× (2.68–2.75) | 2.43× (2.39–2.47) | 1.00× (0.97–1.03) | 1.05× (1.01–1.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.51× (2.48–2.55) | 2.73× (2.70–2.75) | 3.54× (3.43–3.64) | 3.53× (3.50–3.56) | 4.06× (4.01–4.11) | 2.41× (2.39–2.43) | 2.05× (2.03–2.06) | 1.94× (1.91–1.98) | 2.66× (2.63–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.15× (2.12–2.18) | 2.39× (2.32–2.46) | 3.06× (2.92–3.20) | 2.96× (2.90–3.02) | 3.37× (3.30–3.44) | 1.89× (1.85–1.93) | 1.61× (1.58–1.64) | 1.52× (1.50–1.55) | 2.09× (2.04–2.14) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.88–2.10) | 1.70× (1.60–1.81) | 2.17× (2.05–2.31) | 2.03× (1.87–2.19) | 2.10× (1.98–2.23) | 1.00× (0.94–1.06) | 0.85× (0.80–0.90) | 0.80× (0.75–0.86) | 1.10× (1.04–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.34× (1.31–1.37) | 2.66× (2.61–2.71) | 2.40× (2.33–2.48) | 2.45× (2.39–2.50) | 2.86× (2.79–2.93) | 3.13× (3.06–3.19) | 3.18× (3.11–3.25) | 1.22× (1.20–1.25) | 1.36× (1.34–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.29) | 2.51× (2.47–2.55) | 2.20× (2.18–2.22) | 2.89× (2.85–2.94) | 2.59× (2.56–2.61) | 2.77× (2.74–2.80) | 2.82× (2.78–2.85) | 1.08× (1.07–1.10) | 1.20× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.39–1.56) | 2.28× (2.24–2.32) | 1.84× (1.79–1.89) | 2.51× (2.47–2.56) | 2.42× (2.38–2.47) | 2.01× (1.99–2.03) | 2.04× (2.02–2.07) | 0.79× (0.77–0.80) | 0.87× (0.86–0.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.42–1.46) | 3.01× (2.94–3.08) | 2.49× (2.45–2.54) | 2.60× (2.56–2.64) | 3.07× (3.01–3.14) | 3.58× (3.54–3.63) | 3.94× (3.88–3.99) | 1.40× (1.38–1.43) | 1.66× (1.63–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.31–1.36) | 2.72× (2.70–2.75) | 2.26× (2.23–2.29) | 3.12× (3.08–3.16) | 2.71× (2.66–2.76) | 3.09× (3.05–3.13) | 3.39× (3.35–3.44) | 1.21× (1.19–1.23) | 1.43× (1.41–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.45–1.66) | 2.59× (2.47–2.72) | 1.94× (1.83–2.06) | 2.72× (2.65–2.80) | 2.54× (2.46–2.62) | 2.22× (2.17–2.27) | 2.44× (2.39–2.49) | 0.87× (0.85–0.89) | 1.03× (1.00–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.72) | 2.68× (2.62–2.75) | 3.49× (3.38–3.60) | 2.77× (2.70–2.85) | 5.97× (5.78–6.16) | 2.31× (2.25–2.37) | 2.14× (2.09–2.20) | 0.97× (0.95–0.99) | 1.02× (0.96–1.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.65× (0.64–0.67) | 2.50× (2.47–2.54) | 4.25× (4.20–4.30) | 2.69× (2.65–2.74) | 5.55× (5.45–5.64) | 2.14× (2.10–2.18) | 1.98× (1.94–2.03) | 0.90× (0.89–0.91) | 0.95× (0.90–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.61–0.64) | 2.75× (2.70–2.80) | 4.24× (4.19–4.29) | 2.73× (2.70–2.77) | 5.61× (5.52–5.70) | 2.16× (2.13–2.20) | 2.00× (1.97–2.03) | 0.91× (0.90–0.92) | 0.96× (0.91–1.01) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 1.91× (1.87–1.94) | 3.88× (3.80–3.97) | 4.25× (4.16–4.35) | 4.31× (4.21–4.41) | 4.55× (4.44–4.67) | 4.98× (4.87–5.10) | 4.77× (4.66–4.89) | 1.49× (1.45–1.54) | 1.76× (1.72–1.80) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 1.61× (1.58–1.63) | 3.17× (3.10–3.23) | 3.49× (3.39–3.59) | 3.83× (3.76–3.91) | 3.61× (3.50–3.71) | 3.82× (3.72–3.93) | 3.65× (3.57–3.74) | 1.14× (1.11–1.18) | 1.35× (1.32–1.38) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 1.76× (1.71–1.81) | 2.62× (2.51–2.74) | 2.86× (2.71–3.03) | 3.05× (2.93–3.17) | 2.98× (2.86–3.10) | 2.52× (2.47–2.57) | 2.41× (2.36–2.46) | 0.75× (0.73–0.78) | 0.89× (0.86–0.91) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.29× (0.28–0.30) | 1.05× (1.05–1.06) | 0.84× (0.84–0.84) | 11.91× (11.89–11.94) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.37× (0.36–0.37) | 0.57× (0.51–0.63) | 0.76× (0.75–0.77) | 0.64× (0.63–0.64) | 6.04× (6.01–6.07) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 80 MB | 0.38× (0.37–0.38) | 0.70× (0.61–0.80) | 0.68× (0.67–0.69) | 0.59× (0.57–0.60) | 3.93× (3.90–3.96) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.80× (0.80–0.81) | 4.28× (4.28–4.29) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.84× (0.83–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.78) | 0.84× (0.84–0.84) | 4.40× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.04× (8.02–8.06) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.65–0.68) | 0.71× (0.63–0.79) | 1.59× (1.55–1.63) | 0.79× (0.77–0.81) | 5.77× (5.63–5.91) | 0.46× (0.45–0.48) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.73× (0.67–0.79) | 1.13× (1.09–1.18) | 0.69× (0.66–0.73) | 3.32× (3.19–3.45) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.30× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.73) | 5.21× (5.20–5.21) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.50–0.53) | 0.40× (0.36–0.43) | 0.93× (0.91–0.96) | 0.74× (0.72–0.76) | 3.20× (3.11–3.28) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.66) | 0.45× (0.44–0.47) | 0.86× (0.84–0.88) | 0.81× (0.79–0.83) | 2.51× (2.44–2.57) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.82–0.83) | 5.23× (5.21–5.25) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.34× (0.31–0.37) | 0.83× (0.81–0.86) | 0.68× (0.65–0.71) | 2.84× (2.76–2.93) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.61× (0.59–0.62) | 0.50× (0.47–0.52) | 0.80× (0.79–0.81) | 0.80× (0.76–0.83) | 2.30× (2.27–2.33) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.07–1.08) | 0.76× (0.76–0.76) | 5.36× (5.35–5.37) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.76–0.78) | 5.32× (5.30–5.33) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.62× (1.61–1.62) | 0.77× (0.77–0.78) | 5.35× (5.33–5.36) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.43× (1.42–1.43) | 0.66× (0.66–0.66) | 6.43× (6.42–6.44) | 0.39× (0.38–0.39) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.46) | 0.39× (0.36–0.43) | 1.04× (1.00–1.08) | 0.68× (0.66–0.70) | 3.27× (3.19–3.35) | 0.20× (0.19–0.20) | 0.22× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `text` (2.56 MB → 12.8 MB, 5.0×) | 4 | 55 MB | 0.51× (0.50–0.52) | 0.51× (0.49–0.54) | 0.93× (0.91–0.95) | 0.73× (0.69–0.76) | 2.37× (2.30–2.43) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 394 | 2.55 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 636 | 2.96 | 1.16× (1.14–1.18) | 80% | 1.06× (1.04–1.08) | 1.06× (1.04–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 744 | 4.98 | 1.93× (1.90–1.96) | 47% | 1.23× (1.21–1.25) | 1.68× (1.65–1.71) |

**Best:** mibz2 at 4 threads (744 MB/s); lbzip2 at 4 threads (185 MB/s); pbz2 at 4 threads (287 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.17, lbzip2 1t 0.09, lbzip2 2t 0.46, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.82 | 0.02 | 0.08 | 0.282 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.5% |
| MTF + RLE2 | 1.20 | 17.6% |
| T-vector build | 1.15 | 16.9% |
| walk (the one each build uses) | 3.19 | 46.9% |
| of which: segment assembly | 0.11 | 1.7% |
| RLE1 | 0.21 | 3.2% |
| CRC | 0.19 | 2.9% |
| **total** | 6.78 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.493 | 1.304 | (in its total) | 2.65× (2.64–2.65) |  |
| walk | 1.266 | 2.653 | (in its total) | 2.10× (2.09–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.058 | (in its total) | 1.26× (1.24–1.28) |  |
| inverse BWT, total | 1.805 | 4.015 | 3.326 | 2.23× (2.22–2.24) | 1.85× (1.84–1.85) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 8m47s |
| 4 threads | 41m23s ± 5m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 29m58s ± 4m16s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 29m58s ± 4m16s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 3.93 ns a byte of output at 4 threads (CPU, summed over them), 2.05 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,562,767 | 12,847,686 | \`60173b9d9bc7c2e9…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
