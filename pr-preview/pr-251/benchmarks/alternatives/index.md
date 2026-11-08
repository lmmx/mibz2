# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-251/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`e01db6a`](<https://github.com/lmmx/mibz2/commit/e01db6a2895711eb13897fdc378816075d927c1e>) (2026-11-07).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42425892101>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.532 s | 394 | 9.4 | 2.28× (2.26–2.29) | 5.25× (5.20–5.29) | 5.43× (5.40–5.47) | 6.17× (6.12–6.22) | 5.39× (5.35–5.42) | 6.10× (6.06–6.15) | 5.37× (5.35–5.39) | 1.70× (1.69–1.70) | 1.89× (1.88–1.90) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.298 s | 703 | 10.2 | 2.42× (2.33–2.52) | 4.98× (4.74–5.24) | 5.71× (5.48–5.95) | 5.85× (5.66–6.04) | 5.12× (4.94–5.30) | 10.80× (10.43–11.18) | 9.50× (9.20–9.81) | 3.00× (2.91–3.10) | 3.35× (3.24–3.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.250 s | 840 | 16.7 | 2.34× (2.31–2.37) | 3.48× (3.40–3.56) | 4.35× (4.24–4.47) | 4.18× (4.09–4.28) | 4.10× (4.02–4.18) | 13.00× (12.82–13.18) | 11.44× (11.26–11.61) | 3.61× (3.56–3.67) | 4.03× (3.96–4.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,124 | 3.6 | 0.57× (0.56–0.58) | 3.42× (3.39–3.45) | 4.57× (4.53–4.61) | 3.57× (3.53–3.60) | 6.16× (6.09–6.24) | 2.67× (2.65–2.70) | 2.69× (2.62–2.77) | 0.98× (0.96–1.00) | 1.87× (1.69–2.06) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,102 | 3.7 | 0.52× (0.50–0.54) | 2.79× (2.68–2.91) | 5.59× (5.38–5.82) | 3.50× (3.35–3.65) | 5.90× (5.65–6.15) | 2.61× (2.51–2.71) | 2.62× (2.49–2.77) | 0.95× (0.91–1.00) | 1.82× (1.64–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,099 | 3.7 | 0.53× (0.52–0.54) | 2.80× (2.74–2.85) | 5.58× (5.47–5.69) | 3.49× (3.43–3.56) | 5.92× (5.78–6.06) | 2.60× (2.55–2.65) | 2.62× (2.54–2.70) | 0.95× (0.93–0.97) | 1.82× (1.65–2.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 685 | 5.7 | 2.51× (2.50–2.53) | 2.85× (2.84–2.86) | 3.56× (3.55–3.57) | 3.80× (3.78–3.82) | 4.13× (4.10–4.16) | 3.19× (3.17–3.21) | 2.10× (2.09–2.11) | 1.93× (1.92–1.94) | 2.84× (2.83–2.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 s | 1,080 | 6.9 | 2.17× (2.03–2.31) | 2.85× (2.61–3.10) | 4.26× (3.94–4.60) | 3.39× (3.24–3.55) | 4.92× (4.63–5.23) | 4.88× (4.61–5.16) | 3.22× (3.04–3.40) | 2.96× (2.79–3.13) | 4.35× (4.11–4.59) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 912 | 13.8 | 1.78× (1.73–1.83) | 1.85× (1.71–2.00) | 2.85× (2.67–3.04) | 2.05× (1.96–2.14) | 3.83× (3.70–3.96) | 4.24× (4.07–4.41) | 2.79× (2.69–2.90) | 2.57× (2.48–2.66) | 3.77× (3.63–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 s | 73 | 51.5 | 1.36× (1.33–1.40) | 2.63× (2.56–2.70) | 2.39× (2.32–2.46) | 2.49× (2.43–2.56) | 2.76× (2.69–2.84) | 3.14× (3.06–3.23) | 3.06× (2.97–3.14) | 1.24× (1.21–1.28) | 1.40× (1.35–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 s | 119 | 57.7 | 1.30× (1.25–1.34) | 2.51× (2.38–2.65) | 2.38× (2.31–2.46) | 3.12× (3.01–3.23) | 2.87× (2.78–2.97) | 5.15× (5.01–5.29) | 5.01× (4.87–5.15) | 2.03× (1.98–2.09) | 2.29× (2.23–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 s | 144 | 76.6 | 1.36× (1.33–1.39) | 2.16× (2.08–2.23) | 2.25× (2.12–2.39) | 2.63× (2.56–2.69) | 2.91× (2.84–2.99) | 6.30× (6.18–6.41) | 6.12× (6.01–6.24) | 2.49× (2.44–2.53) | 2.81× (2.72–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 82 | 45.9 | 1.53× (1.52–1.54) | 3.10× (3.08–3.12) | 2.68× (2.66–2.70) | 2.83× (2.80–2.86) | 3.12× (3.11–3.14) | 4.02× (4.00–4.05) | 4.02× (4.01–4.02) | 1.52× (1.51–1.52) | 1.80× (1.80–1.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.071 s | 134 | 52.2 | 1.39× (1.36–1.41) | 2.91× (2.79–3.04) | 2.59× (2.52–2.67) | 3.61× (3.51–3.71) | 3.14× (3.06–3.22) | 6.60× (6.45–6.75) | 6.58× (6.44–6.73) | 2.49× (2.43–2.55) | 2.96× (2.90–3.02) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 156 | 72.4 | 1.44× (1.41–1.47) | 2.27× (2.17–2.37) | 2.30× (2.18–2.43) | 3.10× (3.02–3.19) | 2.89× (2.83–2.95) | 7.65× (7.50–7.79) | 7.63× (7.48–7.78) | 2.88× (2.82–2.94) | 3.43× (3.36–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.61–0.63) | 3.45× (3.41–3.48) | 4.33× (4.28–4.37) | 3.09× (3.04–3.13) | 6.73× (6.60–6.86) | 2.33× (2.31–2.35) | 2.31× (2.29–2.33) | 0.95× (0.94–0.96) | 1.73× (1.72–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 s | 813 | 5.3 | 0.52× (0.51–0.53) | 2.50× (2.46–2.53) | 5.05× (4.96–5.14) | 2.99× (2.95–3.03) | 6.46× (6.38–6.55) | 2.24× (2.21–2.27) | 2.22× (2.20–2.24) | 0.91× (0.90–0.92) | 1.67× (1.64–1.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 794 | 5.2 | 0.52× (0.52–0.52) | 2.46× (2.43–2.49) | 4.93× (4.87–4.98) | 2.95× (2.91–2.99) | 6.26× (6.14–6.38) | 2.20× (2.18–2.22) | 2.18× (2.16–2.20) | 0.90× (0.88–0.91) | 1.63× (1.62–1.65) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 0.071 s | 178 | 21.3 | 1.36× (1.34–1.39) | 2.96× (2.91–3.02) | 3.24× (3.18–3.30) | 3.43× (3.36–3.49) | 3.57× (3.50–3.64) | 3.76× (3.69–3.84) | 3.45× (3.39–3.51) | 1.12× (1.10–1.14) | 1.27× (1.24–1.29) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 0.046 s | 277 | 24.9 | 1.17× (1.05–1.31) | 2.45× (2.22–2.70) | 3.02× (2.72–3.35) | 3.41× (3.04–3.83) | 3.44× (3.11–3.80) | 5.70× (5.14–6.32) | 5.22× (4.71–5.79) | 1.69× (1.52–1.88) | 1.92× (1.73–2.13) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 0.037 s | 342 | 36.2 | 1.33× (1.29–1.36) | 2.07× (2.00–2.14) | 2.83× (2.77–2.89) | 3.06× (3.00–3.11) | 3.50× (3.43–3.58) | 7.32× (7.20–7.45) | 6.71× (6.59–6.83) | 2.17× (2.14–2.21) | 2.47× (2.42–2.52) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.27× (2.26–2.29) | 5.22× (5.18–5.26) | 5.41× (5.37–5.45) | 6.16× (6.11–6.20) | 5.37× (5.34–5.41) | 6.08× (6.04–6.13) | 5.35× (5.33–5.37) | 1.69× (1.69–1.70) | 1.88× (1.87–1.89) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.20× (2.18–2.21) | 4.92× (4.86–4.99) | 5.21× (5.17–5.25) | 6.02× (5.98–6.05) | 5.02× (4.98–5.05) | 5.62× (5.56–5.67) | 4.94× (4.89–4.99) | 1.56× (1.55–1.58) | 1.73× (1.71–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.18× (2.15–2.21) | 3.57× (3.53–3.61) | 3.90× (3.84–3.96) | 4.31× (4.26–4.36) | 3.70× (3.66–3.74) | 3.44× (3.41–3.47) | 3.03× (3.00–3.06) | 0.96× (0.95–0.97) | 1.06× (1.05–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.93× (2.90–2.95) | 3.94× (3.85–4.02) | 3.34× (3.31–3.38) | 5.71× (5.64–5.78) | 2.52× (2.50–2.55) | 2.54× (2.47–2.61) | 0.98× (0.96–1.00) | 1.64× (1.62–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.71× (2.62–2.80) | 4.88× (4.69–5.08) | 3.27× (3.15–3.39) | 5.42× (5.23–5.62) | 2.44× (2.36–2.52) | 2.45× (2.34–2.57) | 0.95× (0.91–0.99) | 1.58× (1.53–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.00× (2.94–3.07) | 4.94× (4.85–5.04) | 3.33× (3.28–3.38) | 5.52× (5.40–5.64) | 2.48× (2.44–2.52) | 2.50× (2.43–2.56) | 0.96× (0.94–0.98) | 1.61× (1.58–1.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.43× (2.41–2.44) | 2.64× (2.63–2.65) | 3.14× (3.13–3.16) | 3.64× (3.63–3.66) | 3.95× (3.93–3.97) | 3.06× (3.04–3.08) | 2.04× (2.03–2.05) | 1.88× (1.87–1.89) | 2.47× (2.46–2.48) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.18× (2.17–2.20) | 2.38× (2.32–2.44) | 3.20× (3.17–3.24) | 3.19× (3.15–3.23) | 3.47× (3.43–3.52) | 2.54× (2.51–2.58) | 1.69× (1.68–1.71) | 1.56× (1.55–1.57) | 2.05× (2.03–2.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.85× (1.76–1.94) | 1.59× (1.55–1.64) | 2.08× (2.02–2.13) | 1.95× (1.89–2.01) | 1.95× (1.90–2.01) | 1.25× (1.21–1.29) | 0.83× (0.81–0.86) | 0.77× (0.75–0.79) | 1.01× (0.98–1.04) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.32–1.40) | 2.55× (2.48–2.61) | 2.33× (2.27–2.40) | 2.47× (2.41–2.54) | 2.74× (2.66–2.82) | 3.11× (3.03–3.20) | 3.03× (2.94–3.11) | 1.24× (1.21–1.27) | 1.36× (1.32–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.30) | 2.46× (2.37–2.56) | 2.16× (2.14–2.18) | 2.93× (2.90–2.97) | 2.52× (2.49–2.55) | 2.80× (2.77–2.82) | 2.72× (2.70–2.75) | 1.11× (1.10–1.13) | 1.22× (1.21–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.39–1.51) | 2.31× (2.29–2.34) | 1.96× (1.93–2.00) | 2.65× (2.60–2.69) | 2.53× (2.46–2.61) | 2.13× (2.10–2.16) | 2.07× (2.05–2.10) | 0.85× (0.84–0.86) | 0.93× (0.92–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.53× (1.52–1.53) | 3.01× (2.99–3.02) | 2.61× (2.59–2.62) | 2.81× (2.78–2.83) | 3.09× (3.08–3.11) | 3.98× (3.96–4.00) | 3.97× (3.96–3.98) | 1.51× (1.50–1.52) | 1.74× (1.74–1.75) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.40× (1.39–1.41) | 2.79× (2.76–2.82) | 2.39× (2.36–2.41) | 3.37× (3.32–3.43) | 2.80× (2.77–2.84) | 3.51× (3.48–3.54) | 3.50× (3.47–3.54) | 1.33× (1.32–1.34) | 1.54× (1.53–1.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.63× (1.60–1.67) | 2.57× (2.52–2.62) | 2.03× (2.00–2.07) | 2.91× (2.85–2.97) | 2.65× (2.61–2.69) | 2.52× (2.48–2.57) | 2.52× (2.48–2.56) | 0.96× (0.94–0.97) | 1.11× (1.09–1.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.67) | 2.45× (2.44–2.46) | 3.39× (3.37–3.41) | 2.86× (2.78–2.94) | 6.01× (5.91–6.12) | 2.16× (2.15–2.18) | 2.15× (2.14–2.16) | 0.96× (0.95–0.97) | 1.37× (1.36–1.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.59) | 2.26× (2.24–2.29) | 4.09× (3.97–4.23) | 2.75× (2.72–2.79) | 5.71× (5.63–5.79) | 2.05× (2.02–2.07) | 2.03× (2.01–2.05) | 0.91× (0.90–0.92) | 1.30× (1.27–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.57–0.58) | 2.50× (2.46–2.53) | 4.02× (3.95–4.08) | 2.81× (2.77–2.85) | 5.71× (5.61–5.81) | 2.07× (2.06–2.09) | 2.06× (2.04–2.08) | 0.92× (0.91–0.93) | 1.31× (1.30–1.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 1.35× (1.32–1.38) | 2.83× (2.77–2.89) | 3.13× (3.06–3.20) | 3.37× (3.30–3.44) | 3.50× (3.43–3.58) | 3.69× (3.61–3.77) | 3.38× (3.32–3.45) | 1.11× (1.09–1.13) | 1.21× (1.19–1.24) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 1.26× (1.21–1.31) | 2.57× (2.53–2.61) | 2.85× (2.81–2.88) | 3.33× (3.29–3.37) | 3.11× (3.07–3.14) | 3.19× (3.15–3.24) | 2.93× (2.89–2.97) | 0.96× (0.95–0.98) | 1.05× (1.03–1.06) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 1.38× (1.30–1.47) | 2.15× (2.09–2.21) | 2.42× (2.36–2.48) | 2.74× (2.68–2.80) | 2.63× (2.58–2.68) | 2.19× (2.15–2.23) | 2.01× (1.97–2.04) | 0.66× (0.65–0.67) | 0.72× (0.71–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.33) | 0.36× (0.35–0.38) | 1.08× (1.07–1.08) | 0.86× (0.86–0.86) | 12.18× (12.15–12.20) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.37–0.39) | 0.47× (0.41–0.55) | 0.80× (0.78–0.82) | 0.67× (0.65–0.69) | 6.39× (6.28–6.51) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.40) | 0.54× (0.46–0.65) | 0.71× (0.70–0.72) | 0.62× (0.60–0.64) | 4.12× (4.06–4.18) | 0.09× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.43–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.83–0.84) | 4.42× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.73–1.73) | 0.66× (0.66–0.67) | 8.15× (8.13–8.17) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.67× (0.59–0.75) | 1.64× (1.61–1.66) | 0.80× (0.79–0.81) | 5.93× (5.85–6.02) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.63–0.66) | 0.71× (0.67–0.76) | 1.17× (1.14–1.19) | 0.77× (0.75–0.79) | 3.43× (3.35–3.51) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.74× (0.73–0.74) | 5.25× (5.25–5.26) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.40× (0.36–0.44) | 0.92× (0.88–0.95) | 0.69× (0.64–0.73) | 3.17× (3.05–3.29) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.80× (0.76–0.84) | 2.54× (2.49–2.59) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.83× (0.83–0.84) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.51–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.46–0.50) | 0.36× (0.33–0.40) | 0.87× (0.83–0.90) | 0.72× (0.67–0.76) | 2.97× (2.86–3.08) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.61× (0.59–0.62) | 0.48× (0.46–0.51) | 0.82× (0.80–0.83) | 0.83× (0.81–0.86) | 2.36× (2.31–2.40) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.24× (0.23–0.25) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.44× (5.43–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.78) | 5.40× (5.38–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.20× (0.20–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.39–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.41× (0.40–0.41) | 1.50× (1.50–1.50) | 0.67× (0.66–0.67) | 6.77× (6.76–6.78) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 34 MB | 0.47× (0.45–0.48) | 0.40× (0.36–0.44) | 1.13× (1.09–1.18) | 0.71× (0.69–0.73) | 3.55× (3.45–3.66) | 0.21× (0.20–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 53 MB | 0.52× (0.50–0.55) | 0.49× (0.45–0.53) | 0.97× (0.94–1.00) | 0.77× (0.75–0.80) | 2.47× (2.41–2.54) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 394 | 2.55 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 703 | 2.77 | 1.08× (1.07–1.09) | 88% | 1.02× (1.01–1.03) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 840 | 4.51 | 1.77× (1.75–1.79) | 53% | 1.21× (1.19–1.23) | 1.69× (1.66–1.73) |

**Best:** mibz2 at 4 threads (840 MB/s); lbzip2 at 4 threads (243 MB/s); pbz2 at 4 threads (360 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.70, mibz2 4t 1.06, lbzip2 1t 0.07, lbzip2 2t 0.24, lbzip2 4t 0.35, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.76 | 0.02 | 0.08 | 0.250 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 53.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.3% |
| MTF + RLE2 | 1.58 | 17.5% |
| T-vector build | 1.76 | 19.6% |
| walk (the one each build uses) | 4.08 | 45.3% |
| of which: segment assembly | 0.15 | 1.6% |
| RLE1 | 0.20 | 2.3% |
| CRC | 0.26 | 2.8% |
| **total** | 8.99 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.453 | 1.079 | (in its total) | 2.38× (2.37–2.39) |  |
| walk | 1.048 | 2.054 | (in its total) | 1.96× (1.94–1.97) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.041 | 0.041 | (in its total) | 0.98× (0.93–1.03) |  |
| inverse BWT, total | 1.541 | 3.174 | 1.900 | 2.06× (2.04–2.07) | 1.23× (1.22–1.24) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h17m ± 9m10s |
| 4 threads | 36m29s ± 5m09s |
| 4 threads, the sample's overlap gap as a fixed cost | 27m39s ± 3m53s |
| the scanner alone (its serial part) | 12m22s ± 32.9s |
| the workers alone, over 4 | 27m39s ± 3m53s |

At 4 threads the workers bound it. Per byte: the scanner 7.2 ns a byte of bz2, the workers 3.62 ns a byte of output at 4 threads (CPU, summed over them), 2.14 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,073,075 | 12,677,537 | \`e4ea56361d34c699…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
