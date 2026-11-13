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

The latest record: [`4df9651`](<https://github.com/lmmx/mibz2/commit/4df9651b84d0a01143b601a05dd7748720008fdd>) (2026-11-13).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43398652640>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.538 s | 389 | 8.4 | 2.86× (2.82–2.89) | 6.53× (6.49–6.58) | 6.73× (6.67–6.79) | 7.27× (7.23–7.31) | 6.60× (6.55–6.65) | 7.14× (7.09–7.19) | 6.70× (6.64–6.77) | 2.11× (2.08–2.14) | 2.43× (2.41–2.45) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.307 s | 683 | 9.3 | 2.67× (2.43–2.92) | 5.76× (5.27–6.30) | 6.49× (5.91–7.13) | 6.34× (5.79–6.94) | 5.82× (5.32–6.36) | 11.70× (10.70–12.79) | 10.99× (10.09–11.97) | 3.46× (3.16–3.80) | 3.98× (3.64–4.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.273 s | 767 | 15.5 | 2.65× (2.60–2.70) | 4.01× (3.90–4.12) | 4.97× (4.85–5.10) | 4.53× (4.47–4.59) | 4.61× (4.53–4.69) | 13.97× (13.77–14.17) | 13.12× (12.94–13.29) | 4.14× (4.07–4.20) | 4.75× (4.69–4.80) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 959 | 3.7 | 0.60× (0.59–0.61) | 3.68× (3.49–3.88) | 4.49× (4.28–4.70) | 3.53× (3.46–3.61) | 6.16× (6.05–6.27) | 2.95× (2.89–3.01) | 2.62× (2.57–2.68) | 1.01× (0.99–1.03) | 1.49× (1.45–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 939 | 3.9 | 0.57× (0.55–0.58) | 2.88× (2.81–2.94) | 6.08× (5.92–6.25) | 3.45× (3.37–3.53) | 5.89× (5.77–6.01) | 2.86× (2.79–2.93) | 2.55× (2.49–2.60) | 0.98× (0.95–1.02) | 1.44× (1.41–1.48) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 928 | 3.8 | 0.56× (0.55–0.57) | 2.85× (2.80–2.90) | 6.05× (5.95–6.16) | 3.40× (3.35–3.46) | 5.89× (5.82–5.95) | 2.82× (2.77–2.87) | 2.51× (2.47–2.55) | 0.97× (0.94–1.00) | 1.42× (1.39–1.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 573 | 6.0 | 2.75× (2.62–2.89) | 3.00× (2.86–3.15) | 3.77× (3.67–3.88) | 3.73× (3.63–3.82) | 4.10× (3.96–4.24) | 2.51× (2.44–2.58) | 2.08× (2.02–2.14) | 2.00× (1.93–2.08) | 2.90× (2.76–3.06) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 856 | 7.5 | 2.16× (2.04–2.29) | 2.88× (2.54–3.26) | 4.33× (3.86–4.84) | 3.84× (3.23–4.57) | 4.67× (4.38–4.99) | 3.71× (3.50–3.95) | 3.07× (2.88–3.27) | 2.97× (2.79–3.15) | 4.30× (4.04–4.57) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 750 | 15.0 | 1.82× (1.76–1.88) | 1.87× (1.73–2.01) | 2.97× (2.80–3.15) | 2.19× (2.07–2.31) | 3.85× (3.73–3.96) | 3.32× (3.23–3.41) | 2.75× (2.67–2.83) | 2.65× (2.57–2.73) | 3.84× (3.68–4.00) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.135 s | 59 | 55.7 | 1.33× (1.31–1.36) | 2.64× (2.59–2.69) | 2.41× (2.35–2.47) | 2.39× (2.34–2.45) | 2.79× (2.74–2.83) | 3.07× (3.02–3.12) | 3.11× (3.05–3.16) | 1.20× (1.17–1.23) | 1.36× (1.33–1.40) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.089 s | 90 | 63.1 | 1.15× (1.04–1.27) | 2.23× (2.01–2.48) | 2.08× (1.88–2.31) | 2.55× (2.31–2.81) | 2.58× (2.33–2.86) | 4.48× (4.07–4.93) | 4.54× (4.12–4.99) | 1.76× (1.61–1.91) | 1.99× (1.80–2.20) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 111 | 87.2 | 1.26× (1.22–1.29) | 2.04× (2.01–2.07) | 1.97× (1.91–2.03) | 2.43× (2.29–2.58) | 2.73× (2.70–2.75) | 5.81× (5.73–5.89) | 5.88× (5.79–5.98) | 2.28× (2.22–2.34) | 2.58× (2.52–2.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.149 s | 64 | 51.1 | 1.46× (1.44–1.48) | 3.00× (2.95–3.04) | 2.51× (2.48–2.55) | 2.59× (2.56–2.63) | 3.03× (3.00–3.07) | 3.55× (3.50–3.60) | 3.85× (3.82–3.89) | 1.37× (1.34–1.40) | 1.65× (1.62–1.67) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.090 s | 106 | 57.5 | 1.34× (1.31–1.38) | 2.98× (2.72–3.27) | 2.43× (2.36–2.51) | 3.29× (3.17–3.40) | 3.05× (2.95–3.16) | 5.83× (5.65–6.01) | 6.33× (6.13–6.52) | 2.25× (2.16–2.33) | 2.70× (2.62–2.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 121 | 82.6 | 1.42× (1.37–1.48) | 2.27× (2.12–2.42) | 2.19× (2.11–2.27) | 2.81× (2.72–2.90) | 2.77× (2.69–2.84) | 6.69× (6.50–6.89) | 7.26× (7.05–7.48) | 2.58× (2.50–2.66) | 3.10× (3.00–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 719 | 5.2 | 0.65× (0.64–0.67) | 3.00× (2.83–3.18) | 4.28× (4.18–4.39) | 3.06× (2.99–3.13) | 6.69× (6.55–6.84) | 2.49× (2.44–2.54) | 2.28× (2.24–2.33) | 0.97× (0.94–0.99) | 1.48× (1.45–1.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 678 | 5.6 | 0.56× (0.55–0.57) | 2.80× (2.74–2.87) | 4.92× (4.73–5.12) | 2.95× (2.88–3.02) | 6.41× (6.29–6.53) | 2.37× (2.32–2.42) | 2.17× (2.13–2.22) | 0.92× (0.90–0.94) | 1.41× (1.38–1.44) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 678 | 5.5 | 0.57× (0.56–0.58) | 2.80× (2.74–2.85) | 5.10× (4.88–5.33) | 2.93× (2.88–2.98) | 6.33× (6.21–6.45) | 2.35× (2.31–2.39) | 2.16× (2.12–2.20) | 0.92× (0.90–0.93) | 1.40× (1.38–1.43) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.073 s | 179 | 18.6 | 1.94× (1.91–1.98) | 4.03× (3.94–4.11) | 4.35× (4.23–4.47) | 4.39× (4.32–4.46) | 4.63× (4.56–4.70) | 5.13× (5.06–5.21) | 4.80× (4.71–4.88) | 1.50× (1.47–1.53) | 1.83× (1.78–1.87) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.052 s | 250 | 23.4 | 1.59× (1.40–1.82) | 3.29× (3.13–3.46) | 3.70× (3.40–4.03) | 4.03× (3.77–4.32) | 4.02× (3.71–4.34) | 7.05× (6.52–7.61) | 6.59× (6.09–7.12) | 2.06× (1.89–2.23) | 2.51× (2.32–2.71) | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.048 s | 272 | 35.6 | 1.41× (1.37–1.45) | 2.28× (2.18–2.39) | 3.03× (2.93–3.13) | 3.00× (2.90–3.11) | 3.57× (3.44–3.70) | 7.73× (7.53–7.94) | 7.22× (7.04–7.41) | 2.25× (2.20–2.32) | 2.75× (2.66–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.85× (2.82–2.88) | 6.51× (6.46–6.55) | 6.70× (6.64–6.76) | 7.25× (7.20–7.29) | 6.58× (6.53–6.63) | 7.12× (7.07–7.16) | 6.68× (6.62–6.74) | 2.11× (2.08–2.14) | 2.41× (2.39–2.43) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.68× (2.62–2.74) | 6.02× (5.89–6.16) | 6.29× (6.22–6.37) | 6.83× (6.74–6.94) | 5.98× (5.92–6.04) | 6.38× (6.31–6.45) | 5.99× (5.94–6.04) | 1.89× (1.87–1.91) | 2.16× (2.14–2.18) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.57× (2.54–2.61) | 4.30× (4.19–4.41) | 4.62× (4.54–4.70) | 4.85× (4.77–4.93) | 4.36× (4.29–4.44) | 3.85× (3.80–3.90) | 3.62× (3.57–3.66) | 1.14× (1.13–1.15) | 1.31× (1.29–1.32) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.21× (3.15–3.27) | 4.01× (3.94–4.09) | 3.29× (3.23–3.36) | 5.67× (5.58–5.77) | 2.76× (2.71–2.81) | 2.47× (2.42–2.52) | 1.01× (0.99–1.03) | 1.06× (1.04–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 2.98× (2.92–3.04) | 5.25× (5.15–5.36) | 3.20× (3.14–3.27) | 5.38× (5.28–5.48) | 2.65× (2.59–2.70) | 2.37× (2.32–2.42) | 0.97× (0.94–1.00) | 1.01× (0.99–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.30× (3.26–3.35) | 5.28× (5.18–5.38) | 3.22× (3.17–3.27) | 5.44× (5.38–5.50) | 2.66× (2.62–2.70) | 2.38× (2.34–2.41) | 0.97× (0.95–1.00) | 1.01× (0.99–1.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.65× (2.52–2.78) | 2.77× (2.65–2.89) | 3.44× (3.33–3.55) | 3.57× (3.47–3.66) | 3.92× (3.79–4.05) | 2.42× (2.35–2.50) | 2.01× (1.95–2.07) | 1.95× (1.87–2.02) | 2.63× (2.53–2.73) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.22× (2.17–2.28) | 2.45× (2.40–2.51) | 3.03× (2.94–3.12) | 3.21× (3.14–3.28) | 3.33× (3.24–3.41) | 1.96× (1.91–2.01) | 1.63× (1.59–1.67) | 1.57× (1.53–1.62) | 2.13× (2.08–2.17) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.95× (1.88–2.02) | 1.67× (1.62–1.73) | 2.14× (2.08–2.21) | 2.03× (1.93–2.14) | 2.06× (2.00–2.13) | 0.99× (0.95–1.04) | 0.83× (0.80–0.86) | 0.80× (0.76–0.84) | 1.08× (1.04–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.30–1.35) | 2.57× (2.52–2.62) | 2.34× (2.28–2.40) | 2.37× (2.32–2.43) | 2.76× (2.71–2.81) | 3.03× (2.99–3.09) | 3.07× (3.02–3.13) | 1.20× (1.17–1.23) | 1.33× (1.31–1.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.28) | 2.40× (2.36–2.44) | 2.11× (2.06–2.15) | 2.69× (2.64–2.75) | 2.50× (2.45–2.55) | 2.68× (2.64–2.73) | 2.72× (2.67–2.76) | 1.06× (1.04–1.08) | 1.18× (1.15–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.42× (1.39–1.45) | 2.20× (2.16–2.24) | 1.76× (1.73–1.79) | 2.32× (2.26–2.38) | 2.31× (2.28–2.35) | 1.94× (1.92–1.97) | 1.97× (1.94–2.00) | 0.77× (0.75–0.79) | 0.85× (0.84–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.44–1.48) | 2.93× (2.89–2.97) | 2.45× (2.42–2.48) | 2.57× (2.53–2.60) | 3.00× (2.97–3.04) | 3.52× (3.47–3.56) | 3.81× (3.78–3.85) | 1.36× (1.34–1.39) | 1.62× (1.61–1.64) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.31–1.38) | 2.76× (2.66–2.86) | 2.25× (2.18–2.31) | 3.01× (2.90–3.12) | 2.71× (2.62–2.79) | 3.08× (2.99–3.17) | 3.34× (3.24–3.44) | 1.19× (1.15–1.23) | 1.42× (1.37–1.46) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.55–1.66) | 2.49× (2.42–2.57) | 1.91× (1.85–1.98) | 2.65× (2.59–2.71) | 2.50× (2.45–2.55) | 2.18× (2.13–2.22) | 2.36× (2.32–2.41) | 0.84× (0.83–0.85) | 1.00× (0.99–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.68–0.72) | 2.66× (2.61–2.72) | 3.50× (3.42–3.57) | 2.79× (2.74–2.85) | 5.95× (5.84–6.07) | 2.29× (2.25–2.33) | 2.12× (2.08–2.16) | 0.97× (0.95–1.00) | 0.99× (0.94–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.61–0.64) | 2.47× (2.42–2.52) | 4.25× (4.17–4.33) | 2.72× (2.66–2.77) | 5.65× (5.56–5.74) | 2.15× (2.11–2.19) | 1.99× (1.95–2.03) | 0.91× (0.89–0.94) | 0.93× (0.88–0.97) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.74× (2.68–2.80) | 4.28× (4.21–4.34) | 2.77× (2.72–2.82) | 5.69× (5.59–5.79) | 2.18× (2.14–2.21) | 2.01× (1.98–2.04) | 0.93× (0.91–0.95) | 0.94× (0.90–0.98) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 1.92× (1.88–1.95) | 3.88× (3.82–3.95) | 4.22× (4.13–4.31) | 4.29× (4.23–4.36) | 4.53× (4.47–4.60) | 5.02× (4.95–5.09) | 4.69× (4.61–4.77) | 1.48× (1.45–1.52) | 1.76× (1.73–1.80) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 1.65× (1.60–1.70) | 3.26× (3.18–3.36) | 3.57× (3.47–3.67) | 3.90× (3.80–4.00) | 3.70× (3.61–3.79) | 4.01× (3.91–4.11) | 3.75× (3.64–3.85) | 1.19× (1.15–1.22) | 1.41× (1.37–1.45) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 1.66× (1.60–1.72) | 2.65× (2.58–2.72) | 2.86× (2.80–2.91) | 3.09× (3.02–3.17) | 3.07× (2.97–3.17) | 2.62× (2.56–2.69) | 2.45× (2.39–2.51) | 0.78× (0.75–0.80) | 0.92× (0.89–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.31× (0.30–0.33) | 1.06× (1.05–1.06) | 0.84× (0.84–0.85) | 11.93× (11.91–11.95) | 0.28× (0.28–0.28) | 0.31× (0.31–0.31) | 0.46× (0.46–0.46) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 52 MB | 0.36× (0.36–0.37) | 0.55× (0.49–0.63) | 0.76× (0.75–0.77) | 0.63× (0.63–0.64) | 6.00× (5.97–6.03) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.23–0.23) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 81 MB | 0.38× (0.37–0.38) | 0.63× (0.56–0.72) | 0.67× (0.66–0.68) | 0.59× (0.58–0.60) | 3.91× (3.87–3.95) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.81–0.81) | 4.28× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.66) | 8.03× (8.00–8.06) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.67) | 0.70× (0.65–0.76) | 1.59× (1.55–1.62) | 0.79× (0.78–0.81) | 5.75× (5.62–5.87) | 0.46× (0.45–0.47) | 0.49× (0.48–0.50) | 0.53× (0.52–0.54) | 0.26× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.66) | 0.71× (0.67–0.77) | 1.15× (1.10–1.19) | 0.70× (0.67–0.73) | 3.36× (3.22–3.51) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.73× (0.73–0.74) | 5.22× (5.20–5.23) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.53) | 0.42× (0.39–0.45) | 0.92× (0.89–0.95) | 0.72× (0.67–0.76) | 3.17× (3.06–3.28) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.47) | 0.87× (0.85–0.89) | 0.82× (0.79–0.84) | 2.52× (2.45–2.59) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.41–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.19) | 0.83× (0.82–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.38× (0.35–0.40) | 0.86× (0.83–0.88) | 0.71× (0.69–0.74) | 2.94× (2.85–3.03) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.50× (0.47–0.53) | 0.80× (0.79–0.82) | 0.80× (0.77–0.84) | 2.31× (2.27–2.34) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.21) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.35–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.28× (5.27–5.30) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.61–1.62) | 0.77× (0.77–0.78) | 5.34× (5.32–5.35) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.66) | 6.38× (6.37–6.40) | 0.38× (0.38–0.39) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 34 MB | 0.49× (0.48–0.49) | 0.45× (0.42–0.48) | 1.14× (1.12–1.16) | 0.73× (0.70–0.76) | 3.60× (3.55–3.64) | 0.22× (0.21–0.22) | 0.24× (0.23–0.24) | 0.35× (0.35–0.36) | 0.17× (0.17–0.18) |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 53 MB | 0.52× (0.50–0.53) | 0.54× (0.49–0.58) | 0.95× (0.92–0.99) | 0.78× (0.75–0.81) | 2.42× (2.35–2.50) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 389 | 2.58 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 683 | 2.88 | 1.12× (1.10–1.13) | 82% | 1.03× (1.01–1.05) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 767 | 4.78 | 1.85× (1.82–1.88) | 49% | 1.22× (1.20–1.24) | 1.67× (1.65–1.68) |

**Best:** mibz2 at 4 threads (767 MB/s); lbzip2 at 4 threads (192 MB/s); pbz2 at 4 threads (291 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.78, mibz2 4t 1.16, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.52, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.41. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.79 | 0.02 | 0.09 | 0.273 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 12.3% |
| MTF + RLE2 | 1.59 | 17.2% |
| T-vector build | 1.53 | 16.5% |
| walk (the one each build uses) | 4.40 | 47.7% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.8% |
| **total** | 9.23 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.484 | 1.315 | (in its total) | 2.72× (2.71–2.73) |  |
| walk | 1.265 | 2.694 | (in its total) | 2.12× (2.10–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.048 | 0.056 | (in its total) | 1.16× (1.14–1.18) |  |
| inverse BWT, total | 1.798 | 4.066 | 3.343 | 2.26× (2.24–2.28) | 1.85× (1.84–1.87) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h18m ± 8m52s |
| 4 threads | 40m00s ± 5m38s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m31s ± 4m02s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 28m31s ± 4m02s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 3.74 ns a byte of output at 4 threads (CPU, summed over them), 2.07 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,595,980 | 13,035,883 | \`ca1f7f044bc5b44a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3244.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
