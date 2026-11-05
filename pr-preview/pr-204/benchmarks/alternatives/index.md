# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-204/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`ad8c294`](<https://github.com/lmmx/mibz2/commit/ad8c2945da3e6be8f5b2a153df4e76820efa84eb>) (2026-11-05).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42049324607>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.682 s | 307 | 9.0 | 2.21× (2.18–2.24) | 5.06× (5.01–5.12) | 5.20× (5.14–5.27) | 5.72× (5.64–5.79) | 5.08× (5.02–5.14) | 5.43× (5.38–5.49) | 5.17× (5.12–5.22) | 1.65× (1.63–1.66) | 1.92× (1.90–1.94) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.381 s | 551 | 9.8 | 2.21× (2.11–2.32) | 4.80× (4.64–4.97) | 5.47× (5.24–5.72) | 5.50× (5.19–5.83) | 4.88× (4.66–5.11) | 9.57× (9.14–10.02) | 9.11× (8.70–9.54) | 2.90× (2.77–3.04) | 3.38× (3.23–3.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.310 s | 677 | 15.4 | 2.27× (2.22–2.32) | 3.52× (3.44–3.61) | 4.42× (4.30–4.54) | 3.99× (3.91–4.08) | 3.97× (3.90–4.05) | 11.97× (11.73–12.20) | 11.39× (11.17–11.61) | 3.63× (3.56–3.69) | 4.23× (4.15–4.31) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 955 | 3.2 | 0.59× (0.58–0.60) | 3.86× (3.71–4.03) | 4.38× (4.22–4.54) | 3.51× (3.42–3.60) | 6.05× (5.89–6.21) | 2.77× (2.70–2.85) | 2.61× (2.53–2.68) | 1.01× (0.98–1.04) | 1.48× (1.44–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 932 | 3.3 | 0.56× (0.56–0.57) | 2.89× (2.84–2.94) | 6.12× (5.96–6.28) | 3.46× (3.41–3.52) | 5.93× (5.80–6.07) | 2.72× (2.67–2.76) | 2.55× (2.51–2.60) | 0.99× (0.96–1.02) | 1.45× (1.42–1.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 928 | 3.2 | 0.57× (0.55–0.58) | 2.87× (2.82–2.92) | 6.02× (5.90–6.15) | 3.43× (3.37–3.49) | 5.96× (5.81–6.12) | 2.69× (2.64–2.74) | 2.53× (2.49–2.57) | 0.98× (0.96–1.00) | 1.43× (1.41–1.46) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 538 | 5.4 | 2.44× (2.41–2.48) | 2.81× (2.78–2.83) | 3.76× (3.57–3.96) | 3.65× (3.62–3.68) | 3.86× (3.81–3.90) | 2.32× (2.30–2.34) | 1.95× (1.94–1.97) | 1.88× (1.85–1.90) | 2.79× (2.77–2.82) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 812 | 6.7 | 2.03× (1.90–2.17) | 2.70× (2.39–3.04) | 3.80× (3.53–4.08) | 3.24× (2.94–3.58) | 4.40× (4.16–4.66) | 3.44× (3.23–3.66) | 2.89× (2.72–3.08) | 2.78× (2.61–2.95) | 4.13× (3.88–4.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 727 | 12.4 | 1.72× (1.65–1.79) | 1.85× (1.72–2.00) | 2.86× (2.72–3.01) | 2.11× (1.99–2.24) | 3.69× (3.57–3.82) | 3.15× (3.05–3.26) | 2.65× (2.57–2.74) | 2.54× (2.47–2.62) | 3.79× (3.67–3.92) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.153 s | 52 | 53.3 | 1.17× (1.16–1.18) | 2.32× (2.29–2.35) | 2.10× (2.05–2.16) | 2.13× (2.11–2.14) | 2.44× (2.42–2.46) | 2.67× (2.65–2.69) | 2.75× (2.71–2.78) | 1.08× (1.06–1.10) | 1.21× (1.20–1.23) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.093 s | 86 | 59.4 | 1.11× (1.08–1.15) | 2.20× (2.13–2.28) | 2.06× (1.99–2.12) | 2.71× (2.60–2.83) | 2.57× (2.48–2.66) | 4.34× (4.22–4.47) | 4.47× (4.36–4.59) | 1.75× (1.69–1.82) | 1.98× (1.92–2.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.074 s | 108 | 76.2 | 1.24× (1.21–1.27) | 1.98× (1.93–2.02) | 1.94× (1.88–1.99) | 2.58× (2.44–2.73) | 2.72× (2.60–2.85) | 5.53× (5.44–5.61) | 5.70× (5.60–5.80) | 2.23× (2.18–2.29) | 2.52× (2.44–2.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.203 s | 47 | 58.7 | 1.06× (1.05–1.07) | 2.19× (2.16–2.22) | 1.83× (1.81–1.85) | 1.91× (1.89–1.93) | 2.23× (2.15–2.30) | 2.71× (2.70–2.73) | 2.83× (2.81–2.85) | 1.04× (1.02–1.05) | 1.22× (1.20–1.24) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.120 s | 80 | 64.6 | 1.01× (0.92–1.10) | 2.07× (1.92–2.22) | 1.83× (1.70–1.99) | 2.53× (2.34–2.72) | 2.26× (2.08–2.46) | 4.49× (4.17–4.85) | 4.69× (4.36–5.04) | 1.72× (1.60–1.84) | 2.02× (1.89–2.17) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.093 s | 103 | 84.0 | 1.22× (1.18–1.27) | 1.92× (1.87–1.98) | 1.86× (1.80–1.92) | 2.51× (2.45–2.57) | 2.38× (2.32–2.45) | 6.00× (5.87–6.13) | 6.26× (6.13–6.40) | 2.29× (2.25–2.33) | 2.70× (2.65–2.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 709 | 4.4 | 0.65× (0.65–0.66) | 3.07× (2.88–3.27) | 4.29× (4.24–4.33) | 3.04× (3.00–3.07) | 6.60× (6.52–6.69) | 2.40× (2.37–2.43) | 2.26× (2.24–2.29) | 0.97× (0.95–0.98) | 1.47× (1.46–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 669 | 4.8 | 0.55× (0.53–0.56) | 2.77× (2.72–2.81) | 4.88× (4.72–5.04) | 2.90× (2.85–2.95) | 6.25× (6.13–6.37) | 2.26× (2.23–2.30) | 2.13× (2.10–2.17) | 0.91× (0.89–0.93) | 1.39× (1.37–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 658 | 4.7 | 0.55× (0.54–0.56) | 2.74× (2.71–2.78) | 4.89× (4.70–5.08) | 2.86× (2.83–2.89) | 6.19× (6.11–6.28) | 2.24× (2.20–2.27) | 2.11× (2.08–2.14) | 0.90× (0.88–0.92) | 1.37× (1.36–1.39) | yes |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 1 | 0.081 s | 149 | 18.9 | 1.50× (1.50–1.51) | 3.05× (3.02–3.08) | 3.34× (3.30–3.39) | 3.32× (3.27–3.37) | 3.55× (3.50–3.60) | 3.59× (3.54–3.64) | 3.57× (3.53–3.61) | 1.16× (1.15–1.17) | 1.41× (1.38–1.44) | yes |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 2 | 0.052 s | 231 | 22.4 | 1.28× (1.18–1.39) | 2.63× (2.35–2.93) | 3.07× (2.86–3.30) | 3.34× (3.08–3.63) | 3.34× (3.14–3.56) | 5.36× (5.00–5.75) | 5.33× (4.98–5.72) | 1.74× (1.63–1.85) | 2.10× (1.99–2.23) | yes |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 4 | 0.045 s | 267 | 32.5 | 1.30× (1.23–1.38) | 2.13× (2.00–2.28) | 2.55× (2.46–2.64) | 2.86× (2.77–2.95) | 3.13× (3.02–3.25) | 6.44× (6.24–6.64) | 6.41× (6.21–6.61) | 2.09× (2.01–2.17) | 2.53× (2.40–2.66) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.21× (2.18–2.23) | 5.04× (4.99–5.10) | 5.18× (5.12–5.25) | 5.70× (5.63–5.78) | 5.07× (5.01–5.13) | 5.42× (5.36–5.47) | 5.16× (5.10–5.21) | 1.64× (1.63–1.66) | 1.90× (1.88–1.93) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.12× (2.09–2.14) | 4.82× (4.77–4.87) | 5.05× (5.00–5.10) | 5.57× (5.49–5.65) | 4.77× (4.71–4.83) | 4.97× (4.92–5.01) | 4.73× (4.69–4.77) | 1.51× (1.50–1.52) | 1.75× (1.73–1.76) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.17× (2.15–2.19) | 3.66× (3.61–3.70) | 3.95× (3.90–3.99) | 4.15× (4.09–4.21) | 3.65× (3.61–3.69) | 3.18× (3.14–3.22) | 3.03× (2.99–3.07) | 0.97× (0.95–0.98) | 1.12× (1.11–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.22× (3.14–3.31) | 4.00× (3.89–4.11) | 3.27× (3.19–3.35) | 5.58× (5.44–5.73) | 2.61× (2.54–2.68) | 2.46× (2.39–2.52) | 1.01× (0.98–1.04) | 1.05× (1.02–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 3.00× (2.96–3.04) | 5.30× (5.22–5.37) | 3.22× (3.18–3.27) | 5.43× (5.33–5.53) | 2.53× (2.49–2.56) | 2.38× (2.35–2.41) | 0.98× (0.95–1.00) | 1.02× (0.99–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.62) | 3.32× (3.24–3.41) | 5.30× (5.22–5.37) | 3.25× (3.21–3.30) | 5.53× (5.39–5.68) | 2.55× (2.51–2.59) | 2.40× (2.37–2.44) | 0.99× (0.97–1.01) | 1.02× (1.00–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.38× (2.34–2.41) | 2.61× (2.60–2.62) | 3.43× (3.26–3.62) | 3.52× (3.50–3.54) | 3.72× (3.68–3.75) | 2.26× (2.24–2.27) | 1.91× (1.89–1.92) | 1.84× (1.81–1.86) | 2.51× (2.48–2.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.08× (2.05–2.10) | 2.31× (2.26–2.35) | 2.87× (2.84–2.91) | 2.98× (2.94–3.03) | 3.12× (3.07–3.16) | 1.81× (1.78–1.83) | 1.53× (1.51–1.55) | 1.47× (1.45–1.49) | 2.01× (1.99–2.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.87–2.02) | 1.66× (1.59–1.74) | 2.13× (2.05–2.21) | 2.03× (1.95–2.11) | 2.01× (1.93–2.09) | 0.96× (0.93–1.00) | 0.81× (0.78–0.85) | 0.78× (0.76–0.81) | 1.07× (1.03–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.17× (1.15–1.18) | 2.27× (2.25–2.30) | 2.07× (2.02–2.11) | 2.11× (2.09–2.13) | 2.42× (2.40–2.45) | 2.64× (2.62–2.66) | 2.72× (2.69–2.76) | 1.07× (1.06–1.09) | 1.18× (1.17–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.12× (1.11–1.14) | 2.18× (2.16–2.21) | 1.91× (1.89–1.92) | 2.58× (2.55–2.62) | 2.28× (2.24–2.31) | 2.38× (2.36–2.40) | 2.45× (2.43–2.48) | 0.97× (0.96–0.98) | 1.06× (1.05–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.36× (1.30–1.43) | 2.15× (2.12–2.18) | 1.75× (1.73–1.78) | 2.36× (2.33–2.39) | 2.28× (2.24–2.32) | 1.85× (1.83–1.88) | 1.91× (1.88–1.94) | 0.75× (0.74–0.76) | 0.83× (0.82–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.06× (1.05–1.07) | 2.16× (2.13–2.19) | 1.80× (1.78–1.81) | 1.90× (1.88–1.92) | 2.21× (2.14–2.29) | 2.70× (2.68–2.71) | 2.81× (2.79–2.83) | 1.04× (1.02–1.05) | 1.20× (1.19–1.21) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.01–1.04) | 2.08× (2.05–2.10) | 1.71× (1.70–1.73) | 2.42× (2.39–2.45) | 2.05× (2.03–2.07) | 2.45× (2.42–2.47) | 2.55× (2.54–2.57) | 0.94× (0.93–0.96) | 1.09× (1.08–1.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.34× (1.32–1.37) | 2.07× (2.04–2.09) | 1.60× (1.58–1.63) | 2.30× (2.25–2.35) | 2.08× (2.06–2.11) | 1.89× (1.87–1.91) | 1.97× (1.95–1.99) | 0.73× (0.72–0.74) | 0.84× (0.83–0.85) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.70) | 2.67× (2.64–2.69) | 3.52× (3.47–3.57) | 2.79× (2.76–2.81) | 5.91× (5.84–5.97) | 2.23× (2.20–2.25) | 2.11× (2.09–2.13) | 0.97× (0.95–0.98) | 0.97× (0.94–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.59–0.62) | 2.43× (2.39–2.47) | 4.18× (4.14–4.21) | 2.68× (2.64–2.72) | 5.52× (5.42–5.62) | 2.06× (2.03–2.09) | 1.95× (1.92–1.98) | 0.90× (0.88–0.91) | 0.90× (0.87–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.59–0.61) | 2.72× (2.68–2.76) | 4.23× (4.17–4.30) | 2.73× (2.70–2.75) | 5.62× (5.54–5.69) | 2.09× (2.06–2.12) | 1.98× (1.95–2.01) | 0.91× (0.89–0.93) | 0.91× (0.88–0.94) |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 1 | 1.49× (1.48–1.50) | 2.96× (2.94–2.99) | 3.21× (3.17–3.25) | 3.26× (3.21–3.31) | 3.49× (3.44–3.54) | 3.52× (3.48–3.57) | 3.50× (3.46–3.55) | 1.16× (1.15–1.17) | 1.35× (1.34–1.36) |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 2 | 1.39× (1.32–1.46) | 2.63× (2.60–2.65) | 2.93× (2.91–2.96) | 3.12× (3.08–3.16) | 3.05× (3.02–3.09) | 2.99× (2.95–3.02) | 2.97× (2.94–3.00) | 0.98× (0.97–1.00) | 1.14× (1.13–1.16) |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 4 | 1.50× (1.41–1.59) | 2.27× (2.23–2.31) | 2.48× (2.44–2.52) | 2.60× (2.54–2.65) | 2.57× (2.52–2.62) | 2.06× (2.03–2.09) | 2.05× (2.02–2.09) | 0.68× (0.66–0.69) | 0.79× (0.77–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.35) | 1.08× (1.08–1.09) | 0.86× (0.86–0.86) | 12.24× (12.21–12.26) | 0.28× (0.28–0.29) | 0.31× (0.31–0.31) | 0.47× (0.47–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.38–0.38) | 0.66× (0.59–0.73) | 0.81× (0.80–0.81) | 0.67× (0.66–0.68) | 6.43× (6.40–6.46) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.71× (0.62–0.81) | 0.71× (0.70–0.72) | 0.62× (0.61–0.64) | 4.11× (4.08–4.14) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.83× (0.82–0.83) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.84–0.84) | 4.45× (4.45–4.45) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.74–1.74) | 0.66× (0.66–0.66) | 8.19× (8.17–8.21) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.75–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.69) | 0.73× (0.68–0.77) | 1.66× (1.61–1.71) | 0.81× (0.78–0.84) | 5.97× (5.79–6.16) | 0.47× (0.46–0.49) | 0.50× (0.48–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.59–0.65) | 0.70× (0.64–0.75) | 1.14× (1.09–1.20) | 0.73× (0.69–0.77) | 3.33× (3.17–3.49) | 0.25× (0.24–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.33) | 1.20× (1.20–1.20) | 0.74× (0.73–0.74) | 5.28× (5.27–5.29) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.54) | 0.44× (0.43–0.45) | 0.94× (0.91–0.97) | 0.73× (0.69–0.76) | 3.25× (3.15–3.35) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.64) | 0.45× (0.45–0.46) | 0.86× (0.85–0.87) | 0.79× (0.76–0.81) | 2.50× (2.47–2.53) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.20× (1.20–1.21) | 0.83× (0.83–0.84) | 5.29× (5.28–5.31) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.46× (0.45–0.47) | 0.37× (0.35–0.40) | 0.83× (0.81–0.85) | 0.69× (0.67–0.71) | 2.84× (2.77–2.91) | 0.17× (0.17–0.18) | 0.18× (0.18–0.19) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.61) | 0.50× (0.48–0.53) | 0.81× (0.81–0.82) | 0.81× (0.78–0.84) | 2.34× (2.31–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 22 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.22) | 1.10× (1.10–1.10) | 0.77× (0.77–0.77) | 5.47× (5.46–5.49) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.66) | 0.77× (0.77–0.77) | 5.45× (5.43–5.47) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.66× (1.65–1.66) | 0.78× (0.77–0.78) | 5.47× (5.46–5.48) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.40–0.41) | 1.52× (1.51–1.52) | 0.65× (0.64–0.65) | 6.81× (6.79–6.83) | 0.41× (0.40–0.41) | 0.43× (0.43–0.44) | 0.66× (0.65–0.67) | 0.33× (0.33–0.34) |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 2 | 30 MB | 0.53× (0.52–0.54) | 0.43× (0.41–0.45) | 1.32× (1.28–1.36) | 0.79× (0.75–0.82) | 4.10× (4.04–4.16) | 0.24× (0.24–0.25) | 0.26× (0.25–0.27) | 0.40× (0.39–0.40) | 0.20× (0.20–0.21) |
| `text` (1.65 MB → 12.1 MB, 7.3×) | 4 | 49 MB | 0.55× (0.53–0.56) | 0.58× (0.53–0.63) | 1.04× (1.02–1.06) | 0.82× (0.79–0.85) | 2.66× (2.60–2.71) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 307 | 3.26 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 551 | 3.58 | 1.09× (1.07–1.11) | 88% | 1.04× (1.03–1.05) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 677 | 5.59 | 1.70× (1.67–1.73) | 55% | 1.23× (1.22–1.25) | 1.67× (1.65–1.69) |

**Best:** mibz2 at 4 threads (677 MB/s); lbzip2 at 4 threads (193 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.66, mibz2 4t 1.10, lbzip2 1t 0.09, lbzip2 2t 0.42, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.31, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 0.94 | 0.02 | 0.09 | 0.310 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2748 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m07s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.04 | 12.2% |
| MTF + RLE2 | 1.68 | 19.7% |
| T-vector build | 1.81 | 21.2% |
| walk (the one each build uses) | 3.52 | 41.3% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.25 | 2.9% |
| CRC | 0.22 | 2.6% |
| **total** | 8.51 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.675 | 1.342 | (in its total) | 1.99× (1.98–2.00) |  |
| walk | 1.217 | 2.659 | (in its total) | 2.19× (2.17–2.21) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.056 | (in its total) | 1.21× (1.18–1.24) |  |
| inverse BWT, total | 1.938 | 4.057 | 3.320 | 2.10× (2.08–2.11) | 1.71× (1.70–1.73) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h39m ± 11m36s |
| 4 threads | 45m20s ± 6m25s |
| 4 threads, the sample's overlap gap as a fixed cost | 34m16s ± 4m50s |
| the scanner alone (its serial part) | 17m19s ± 19.7s |
| the workers alone, over 4 | 34m16s ± 4m50s |

At 4 threads the workers bound it. Per byte: the scanner 10.0 ns a byte of bz2, the workers 4.49 ns a byte of output at 4 threads (CPU, summed over them), 2.71 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,652,427 | 12,115,271 | \`2875e1c65a6ae12a…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2748.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu26 20261026.180.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4
