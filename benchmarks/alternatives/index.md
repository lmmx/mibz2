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

The latest record: [`f6493ca`](<https://github.com/lmmx/mibz2/commit/f6493ca0964662d92b798bf44a125763c0453e3e>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44890465813>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.400 s | 525 | 7.1 | 3.01× (2.92–3.10) | 6.96× (6.72–7.20) | 7.19× (6.97–7.41) | 8.15× (7.91–8.40) | 7.18× (6.96–7.42) | 7.50× (7.29–7.72) | 7.04× (6.84–7.24) | 2.23× (2.16–2.29) | 2.51× (2.43–2.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.241 s | 870 | 7.8 | 3.00× (2.83–3.17) | 6.14× (5.72–6.59) | 6.99× (6.70–7.30) | 7.31× (6.93–7.71) | 6.45× (6.16–6.76) | 12.56× (11.96–13.18) | 11.78× (11.19–12.39) | 3.72× (3.54–3.91) | 4.19× (3.99–4.41) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.197 s | 1,066 | 12.1 | 2.93× (2.87–3.00) | 4.33× (4.21–4.46) | 5.44× (5.25–5.64) | 5.44× (5.28–5.61) | 5.27× (5.16–5.38) | 15.44× (15.09–15.81) | 14.48× (14.17–14.81) | 4.58× (4.48–4.69) | 5.16× (5.04–5.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,111 | 3.6 | 0.57× (0.56–0.58) | 3.41× (3.35–3.47) | 4.61× (4.48–4.75) | 3.56× (3.51–3.61) | 6.10× (5.98–6.23) | 2.65× (2.60–2.70) | 2.65× (2.60–2.69) | 0.97× (0.93–1.02) | 2.08× (1.90–2.28) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,108 | 3.7 | 0.53× (0.51–0.54) | 2.81× (2.74–2.88) | 5.64× (5.50–5.78) | 3.58× (3.46–3.70) | 5.98× (5.85–6.11) | 2.61× (2.56–2.68) | 2.61× (2.55–2.68) | 0.96× (0.93–0.99) | 2.06× (1.85–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,070 | 3.8 | 0.52× (0.47–0.58) | 2.67× (2.47–2.89) | 5.42× (5.20–5.66) | 3.33× (3.09–3.60) | 5.74× (5.29–6.21) | 2.47× (2.28–2.68) | 2.47× (2.28–2.67) | 0.91× (0.83–0.99) | 1.95× (1.66–2.28) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 738 | 5.3 | 2.68× (2.63–2.74) | 3.26× (3.00–3.53) | 3.81× (3.73–3.88) | 4.10× (4.00–4.19) | 4.48× (4.39–4.56) | 2.76× (2.70–2.83) | 2.25× (2.21–2.30) | 2.07× (2.03–2.11) | 3.08× (2.96–3.21) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,098 | 6.5 | 2.18× (1.97–2.41) | 2.78× (2.53–3.06) | 4.15× (3.70–4.65) | 3.45× (3.17–3.75) | 4.98× (4.53–5.48) | 4.01× (3.63–4.42) | 3.27× (2.98–3.60) | 3.00× (2.75–3.28) | 4.47× (4.01–4.98) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 952 | 13.2 | 1.90× (1.85–1.95) | 1.87× (1.71–2.03) | 2.90× (2.71–3.09) | 2.15× (2.07–2.24) | 4.09× (3.94–4.26) | 3.63× (3.56–3.71) | 2.96× (2.90–3.03) | 2.72× (2.67–2.77) | 4.05× (3.92–4.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.4 | 1.34× (1.23–1.46) | 2.56× (2.34–2.79) | 2.35× (2.13–2.59) | 2.43× (2.22–2.65) | 2.72× (2.49–2.98) | 2.92× (2.66–3.21) | 2.91× (2.68–3.17) | 1.21× (1.11–1.33) | 1.34× (1.22–1.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.070 s | 114 | 58.9 | 1.28× (1.21–1.35) | 2.44× (2.32–2.55) | 2.25× (2.16–2.34) | 2.96× (2.83–3.09) | 2.82× (2.69–2.95) | 4.69× (4.46–4.94) | 4.68× (4.46–4.92) | 1.95× (1.87–2.03) | 2.16× (2.06–2.27) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 145 | 76.9 | 1.35× (1.31–1.38) | 2.14× (2.07–2.22) | 2.18× (2.11–2.25) | 2.68× (2.62–2.74) | 2.97× (2.89–3.05) | 6.02× (5.82–6.22) | 6.00× (5.89–6.12) | 2.50× (2.43–2.56) | 2.77× (2.70–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 46.2 | 1.52× (1.50–1.53) | 3.04× (2.99–3.10) | 2.67× (2.64–2.70) | 2.76× (2.74–2.79) | 3.15× (3.08–3.21) | 3.71× (3.67–3.75) | 3.92× (3.86–3.98) | 1.52× (1.49–1.56) | 1.82× (1.79–1.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 s | 131 | 53.3 | 1.36× (1.31–1.40) | 2.94× (2.62–3.29) | 2.51× (2.45–2.56) | 3.72× (3.41–4.07) | 3.07× (3.00–3.14) | 5.99× (5.87–6.12) | 6.33× (6.18–6.49) | 2.46× (2.39–2.54) | 2.94× (2.87–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.064 s | 151 | 75.2 | 1.38× (1.32–1.45) | 2.18× (2.10–2.27) | 2.20× (2.08–2.34) | 2.98× (2.80–3.17) | 2.79× (2.66–2.91) | 6.82× (6.52–7.13) | 7.21× (6.92–7.50) | 2.80× (2.65–2.96) | 3.35× (3.23–3.48) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.61–0.63) | 3.43× (3.38–3.48) | 4.31× (4.25–4.37) | 3.09× (3.06–3.13) | 6.67× (6.58–6.77) | 2.26× (2.23–2.30) | 2.29× (2.26–2.33) | 0.96× (0.93–0.99) | 1.73× (1.70–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 775 | 5.5 | 0.50× (0.49–0.51) | 2.39× (2.34–2.44) | 4.90× (4.65–5.16) | 2.86× (2.80–2.93) | 6.13× (5.97–6.29) | 2.09× (2.05–2.14) | 2.12× (2.08–2.17) | 0.89× (0.86–0.92) | 1.59× (1.56–1.63) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 5.4 | 0.52× (0.49–0.54) | 2.41× (2.37–2.44) | 4.82× (4.75–4.89) | 2.88× (2.84–2.92) | 6.26× (6.08–6.46) | 2.10× (2.06–2.13) | 2.13× (2.10–2.16) | 0.89× (0.87–0.92) | 1.60× (1.57–1.62) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.058 s | 230 | 16.6 | 1.82× (1.81–1.83) | 4.02× (3.96–4.08) | 4.46× (4.34–4.58) | 4.63× (4.56–4.71) | 4.79× (4.71–4.86) | 4.77× (4.63–4.91) | 4.65× (4.53–4.76) | 1.51× (1.50–1.52) | 1.74× (1.73–1.75) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.040 s | 335 | 20.4 | 1.54× (1.35–1.76) | 3.38× (3.05–3.75) | 3.92× (3.79–4.05) | 4.34× (4.12–4.57) | 4.37× (4.22–4.52) | 6.90× (6.56–7.25) | 6.72× (6.44–7.00) | 2.18× (2.10–2.26) | 2.52× (2.43–2.61) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.037 s | 364 | 31.7 | 1.41× (1.20–1.66) | 2.19× (1.88–2.55) | 3.01× (2.59–3.50) | 3.09× (2.68–3.57) | 3.72× (3.15–4.40) | 7.16× (6.17–8.30) | 6.97× (6.01–8.07) | 2.26× (1.96–2.61) | 2.61× (2.26–3.02) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.00× (2.91–3.09) | 6.92× (6.69–7.16) | 7.15× (6.93–7.37) | 8.12× (7.88–8.37) | 7.16× (6.93–7.39) | 7.47× (7.26–7.69) | 7.01× (6.82–7.21) | 2.22× (2.15–2.29) | 2.49× (2.41–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.89× (2.84–2.95) | 6.47× (6.39–6.56) | 6.77× (6.65–6.88) | 7.85× (7.78–7.92) | 6.60× (6.54–6.66) | 6.84× (6.78–6.90) | 6.41× (6.35–6.48) | 2.03× (2.01–2.05) | 2.27× (2.25–2.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.92× (2.87–2.98) | 4.82× (4.67–4.97) | 5.30× (5.10–5.51) | 6.01× (5.87–6.15) | 5.12× (5.02–5.23) | 4.39× (4.26–4.52) | 4.11× (4.00–4.23) | 1.30× (1.26–1.34) | 1.46× (1.42–1.50) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.92× (2.88–2.96) | 3.94× (3.82–4.06) | 3.34× (3.30–3.38) | 5.66× (5.56–5.77) | 2.51× (2.47–2.55) | 2.51× (2.47–2.54) | 0.98× (0.94–1.02) | 1.64× (1.62–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.56–0.59) | 2.71× (2.65–2.78) | 4.93× (4.84–5.03) | 3.33× (3.21–3.44) | 5.47× (5.36–5.59) | 2.44× (2.38–2.49) | 2.43× (2.37–2.49) | 0.95× (0.92–0.98) | 1.60× (1.55–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.51–0.62) | 2.86× (2.65–3.09) | 4.85× (4.62–5.09) | 3.16× (2.94–3.41) | 5.35× (4.94–5.78) | 2.35× (2.18–2.54) | 2.35× (2.18–2.54) | 0.92× (0.84–1.00) | 1.54× (1.43–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.58× (2.54–2.63) | 2.97× (2.74–3.23) | 3.35× (3.29–3.41) | 3.91× (3.82–3.99) | 4.26× (4.19–4.34) | 2.65× (2.59–2.71) | 2.18× (2.14–2.22) | 2.00× (1.97–2.04) | 2.69× (2.54–2.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.27× (2.23–2.32) | 2.53× (2.46–2.61) | 3.32× (3.25–3.40) | 3.33× (3.27–3.40) | 3.63× (3.57–3.70) | 2.17× (2.11–2.23) | 1.78× (1.74–1.81) | 1.64× (1.61–1.67) | 2.20× (2.06–2.34) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.90–2.11) | 1.72× (1.64–1.80) | 2.23× (2.15–2.32) | 2.05× (1.97–2.14) | 2.14× (2.07–2.21) | 1.10× (1.05–1.14) | 0.90× (0.87–0.93) | 0.83× (0.80–0.85) | 1.11× (1.03–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.22–1.45) | 2.49× (2.29–2.72) | 2.30× (2.08–2.54) | 2.41× (2.20–2.63) | 2.70× (2.46–2.95) | 2.89× (2.62–3.18) | 2.88× (2.65–3.14) | 1.21× (1.10–1.32) | 1.32× (1.21–1.45) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.23–1.29) | 2.39× (2.34–2.44) | 2.11× (2.06–2.16) | 2.87× (2.77–2.97) | 2.52× (2.41–2.63) | 2.59× (2.53–2.66) | 2.59× (2.53–2.65) | 1.08× (1.05–1.12) | 1.19× (1.16–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.44–1.52) | 2.31× (2.23–2.39) | 1.97× (1.92–2.01) | 2.66× (2.61–2.71) | 2.53× (2.48–2.58) | 2.02× (1.97–2.07) | 2.01× (1.99–2.04) | 0.84× (0.83–0.86) | 0.92× (0.91–0.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.49–1.52) | 2.97× (2.93–3.01) | 2.60× (2.56–2.64) | 2.73× (2.70–2.76) | 3.11× (3.05–3.18) | 3.66× (3.62–3.71) | 3.87× (3.81–3.94) | 1.51× (1.48–1.55) | 1.76× (1.72–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.35–1.41) | 2.82× (2.66–3.00) | 2.33× (2.29–2.37) | 3.44× (3.19–3.72) | 2.75× (2.70–2.80) | 3.19× (3.13–3.25) | 3.37× (3.30–3.44) | 1.32× (1.28–1.36) | 1.53× (1.49–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.55× (1.48–1.63) | 2.45× (2.35–2.55) | 1.97× (1.87–2.07) | 2.78× (2.64–2.93) | 2.53× (2.42–2.66) | 2.24× (2.13–2.35) | 2.37× (2.26–2.47) | 0.92× (0.88–0.98) | 1.07× (1.02–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.49× (2.41–2.57) | 3.37× (3.32–3.42) | 2.83× (2.81–2.86) | 5.97× (5.89–6.04) | 2.11× (2.08–2.13) | 2.13× (2.11–2.15) | 0.97× (0.94–0.99) | 1.37× (1.35–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.57× (0.56–0.59) | 2.19× (2.15–2.23) | 4.00× (3.77–4.24) | 2.66× (2.61–2.71) | 5.44× (5.31–5.57) | 1.92× (1.89–1.96) | 1.95× (1.91–1.98) | 0.88× (0.86–0.91) | 1.25× (1.22–1.28) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.55–0.60) | 2.45× (2.42–2.49) | 3.92× (3.87–3.98) | 2.74× (2.70–2.77) | 5.71× (5.54–5.88) | 1.98× (1.95–2.01) | 2.01× (1.98–2.04) | 0.91× (0.89–0.93) | 1.29× (1.26–1.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.79× (1.78–1.81) | 3.79× (3.74–3.85) | 4.25× (4.13–4.38) | 4.52× (4.45–4.59) | 4.67× (4.61–4.73) | 4.65× (4.53–4.77) | 4.53× (4.42–4.64) | 1.49× (1.48–1.51) | 1.64× (1.61–1.66) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.52× (1.47–1.57) | 3.16× (3.06–3.27) | 3.53× (3.43–3.64) | 4.13× (4.00–4.27) | 3.82× (3.74–3.90) | 3.73× (3.56–3.89) | 3.63× (3.50–3.76) | 1.20× (1.16–1.23) | 1.31× (1.27–1.36) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.56× (1.33–1.84) | 2.38× (2.06–2.74) | 2.74× (2.36–3.18) | 3.06× (2.65–3.54) | 3.02× (2.52–3.62) | 2.29× (1.97–2.66) | 2.23× (1.93–2.58) | 0.73× (0.64–0.85) | 0.81× (0.70–0.93) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.33× (0.32–0.33) | 0.97× (0.97–0.98) | 0.78× (0.78–0.79) | 10.99× (10.94–11.04) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.34) | 0.45× (0.38–0.53) | 0.66× (0.64–0.69) | 0.56× (0.54–0.58) | 5.32× (5.11–5.54) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 93 MB | 0.33× (0.31–0.34) | 0.47× (0.39–0.57) | 0.58× (0.56–0.60) | 0.51× (0.50–0.53) | 3.39× (3.27–3.52) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.28× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.84) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.83–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.43–0.44) | 1.69× (1.68–1.69) | 0.66× (0.65–0.67) | 7.95× (7.91–8.00) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.36× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.63–0.68) | 0.66× (0.62–0.70) | 1.56× (1.51–1.61) | 0.78× (0.75–0.81) | 5.66× (5.47–5.86) | 0.46× (0.44–0.48) | 0.48× (0.47–0.50) | 0.53× (0.51–0.54) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.70× (0.68–0.73) | 1.14× (1.11–1.18) | 0.70× (0.68–0.72) | 3.37× (3.29–3.46) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.16× (1.16–1.16) | 0.73× (0.73–0.73) | 5.15× (5.14–5.16) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.51) | 0.39× (0.37–0.42) | 0.88× (0.85–0.91) | 0.70× (0.67–0.74) | 3.06× (2.96–3.16) | 0.19× (0.18–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.87× (0.85–0.88) | 0.82× (0.79–0.84) | 2.52× (2.48–2.56) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.83× (0.83–0.83) | 5.18× (5.16–5.20) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 44 MB | 0.46× (0.44–0.48) | 0.35× (0.32–0.38) | 0.81× (0.79–0.83) | 0.69× (0.67–0.71) | 2.79× (2.73–2.85) | 0.17× (0.17–0.18) | 0.19× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.44–0.51) | 0.80× (0.79–0.80) | 0.78× (0.75–0.82) | 2.30× (2.28–2.32) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.34× (5.33–5.36) | 0.35× (0.34–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.59) | 0.76× (0.76–0.76) | 5.26× (5.24–5.28) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.67× (0.67–0.68) | 6.38× (6.36–6.39) | 0.39× (0.39–0.39) | 0.42× (0.42–0.42) | 0.63× (0.63–0.63) | 0.31× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.47) | 0.38× (0.35–0.41) | 1.04× (1.01–1.09) | 0.69× (0.67–0.71) | 3.35× (3.25–3.45) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.52) | 0.44× (0.41–0.48) | 0.90× (0.88–0.93) | 0.76× (0.74–0.78) | 2.31× (2.25–2.37) | 0.13× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 525 | 1.91 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 870 | 2.11 | 1.09× (1.07–1.12) | 84% | 1.02× (1.00–1.04) | 1.05× (1.03–1.08) |
| 4: all 4 CPUs, sharing SMT siblings | 1,066 | 3.26 | 1.70× (1.64–1.77) | 51% | 1.19× (1.17–1.21) | 1.66× (1.64–1.69) |

**Best:** mibz2 at 4 threads (1,066 MB/s); lbzip2 at 4 threads (246 MB/s); pbz2 at 4 threads (367 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.23, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.39, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 11%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.52 | 0.02 | 0.08 | 0.197 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3700 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.7s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.9% |
| MTF + RLE2 | 1.53 | 17.7% |
| T-vector build | 1.50 | 17.4% |
| walk (the one each build uses) | 4.04 | 46.9% |
| of which: segment assembly | 0.14 | 1.7% |
| RLE1 | 0.19 | 2.2% |
| CRC | 0.26 | 3.0% |
| **total** | 8.62 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.391 | 1.062 | (in its total) | 2.72× (2.71–2.73) |  |
| walk | 1.041 | 2.043 | (in its total) | 1.96× (1.95–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.037 | 0.041 | (in its total) | 1.11× (1.09–1.12) |  |
| inverse BWT, total | 1.469 | 3.144 | 1.904 | 2.14× (2.13–2.15) | 1.30× (1.29–1.31) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 58m52s ± 6m52s |
| 4 threads | 28m36s ± 4m04s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m59s ± 2m46s |
| the scanner alone (its serial part) | 11m30s ± 19.7s |
| the workers alone, over 4 | 18m59s ± 2m46s |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 2.49 ns a byte of output at 4 threads (CPU, summed over them), 1.55 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,633,100 | 13,390,258 | \`b7ebdafdab81f6ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3700.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
