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

The latest record: [`848dd3b`](<https://github.com/lmmx/mibz2/commit/848dd3bf01d5cea751697535c91d8b261d2790b6>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42568459642>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.651 s | 322 | 10.1 | 2.34× (2.33–2.35) | 5.40× (5.38–5.42) | 5.57× (5.54–5.60) | 6.02× (6.00–6.05) | 5.43× (5.40–5.45) | 5.81× (5.78–5.84) | 5.43× (5.40–5.46) | 1.76× (1.75–1.77) | 2.03× (2.02–2.04) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.374 s | 561 | 10.9 | 2.40× (2.30–2.50) | 5.23× (5.03–5.44) | 5.74× (5.64–5.85) | 5.71× (5.60–5.82) | 5.14× (4.97–5.32) | 10.18× (9.91–10.46) | 9.51× (9.26–9.77) | 3.08× (3.00–3.16) | 3.56× (3.47–3.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.301 s | 695 | 17.5 | 2.39× (2.35–2.43) | 3.71× (3.62–3.79) | 4.62× (4.51–4.74) | 4.20× (4.12–4.28) | 4.24× (4.14–4.35) | 12.58× (12.36–12.81) | 11.76× (11.54–11.97) | 3.80× (3.74–3.87) | 4.40× (4.31–4.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 976 | 3.6 | 0.60× (0.59–0.60) | 3.74× (3.56–3.93) | 4.46× (4.37–4.54) | 3.58× (3.52–3.63) | 6.17× (6.07–6.27) | 2.83× (2.79–2.87) | 2.66× (2.62–2.70) | 1.02× (1.00–1.03) | 1.51× (1.48–1.53) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 962 | 3.8 | 0.57× (0.56–0.58) | 2.96× (2.90–3.02) | 5.97× (5.73–6.22) | 3.54× (3.46–3.63) | 6.01× (5.90–6.11) | 2.78× (2.72–2.84) | 2.61× (2.55–2.67) | 1.00× (0.97–1.03) | 1.48× (1.45–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 952 | 3.7 | 0.56× (0.55–0.57) | 2.93× (2.88–2.99) | 5.85× (5.66–6.05) | 3.48× (3.42–3.55) | 6.02× (5.89–6.14) | 2.74× (2.69–2.80) | 2.58× (2.53–2.63) | 0.99× (0.97–1.01) | 1.46× (1.43–1.49) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 541 | 6.3 | 2.46× (2.43–2.50) | 2.82× (2.79–2.85) | 3.62× (3.58–3.65) | 3.56× (3.54–3.59) | 3.87× (3.83–3.91) | 2.33× (2.31–2.35) | 1.97× (1.96–1.99) | 1.90× (1.89–1.90) | 2.81× (2.78–2.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 820 | 7.8 | 2.07× (2.02–2.14) | 2.63× (2.45–2.83) | 3.82× (3.63–4.03) | 3.19× (3.00–3.41) | 4.43× (4.23–4.64) | 3.48× (3.33–3.63) | 2.95× (2.83–3.07) | 2.83× (2.71–2.96) | 4.20× (4.02–4.38) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 736 | 15.1 | 1.79× (1.71–1.87) | 1.73× (1.59–1.89) | 2.81× (2.65–2.98) | 2.08× (1.98–2.18) | 3.66× (3.53–3.80) | 3.12× (3.01–3.24) | 2.64× (2.55–2.74) | 2.54× (2.45–2.64) | 3.76× (3.63–3.91) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 53.3 | 1.36× (1.35–1.37) | 2.76× (2.72–2.79) | 2.53× (2.49–2.58) | 2.52× (2.50–2.54) | 2.91× (2.88–2.94) | 3.14× (3.12–3.17) | 3.15× (3.13–3.17) | 1.27× (1.26–1.28) | 1.41× (1.39–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 100 | 60.4 | 1.29× (1.24–1.34) | 2.53× (2.44–2.62) | 2.40× (2.32–2.48) | 3.07× (2.99–3.14) | 2.96× (2.87–3.04) | 5.05× (4.87–5.24) | 5.07× (4.89–5.25) | 2.05× (1.97–2.13) | 2.27× (2.19–2.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 s | 117 | 81.9 | 1.31× (1.27–1.34) | 2.19× (2.12–2.27) | 2.08× (2.00–2.15) | 2.59× (2.52–2.65) | 2.86× (2.80–2.93) | 5.97× (5.88–6.06) | 5.99× (5.90–6.08) | 2.42× (2.39–2.45) | 2.69× (2.64–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 49.2 | 1.45× (1.44–1.47) | 3.09× (3.07–3.12) | 2.63× (2.59–2.67) | 2.69× (2.66–2.73) | 3.16× (3.06–3.26) | 3.78× (3.76–3.80) | 3.88× (3.85–3.92) | 1.46× (1.43–1.49) | 1.71× (1.68–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 56.2 | 1.34× (1.32–1.36) | 2.84× (2.76–2.93) | 2.48× (2.42–2.53) | 3.44× (3.38–3.49) | 3.12× (3.07–3.17) | 6.22× (6.13–6.31) | 6.39× (6.28–6.50) | 2.40× (2.33–2.47) | 2.81× (2.74–2.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 122 | 83.5 | 1.36× (1.31–1.41) | 2.22× (2.15–2.29) | 2.17× (2.13–2.20) | 2.94× (2.88–3.00) | 2.77× (2.72–2.82) | 6.92× (6.81–7.03) | 7.11× (6.98–7.24) | 2.67× (2.61–2.73) | 3.13× (3.06–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 725 | 5.2 | 0.66× (0.65–0.66) | 2.97× (2.95–2.99) | 4.35× (4.32–4.38) | 3.07× (3.07–3.08) | 6.71× (6.65–6.77) | 2.43× (2.42–2.45) | 2.29× (2.27–2.31) | 0.97× (0.96–0.98) | 1.50× (1.49–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 685 | 5.5 | 0.56× (0.55–0.57) | 2.85× (2.80–2.90) | 4.89× (4.81–4.97) | 2.97× (2.92–3.03) | 6.44× (6.32–6.56) | 2.33× (2.29–2.37) | 2.19× (2.16–2.23) | 0.93× (0.91–0.95) | 1.43× (1.41–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 680 | 5.4 | 0.56× (0.54–0.57) | 2.83× (2.77–2.88) | 4.97× (4.76–5.18) | 2.94× (2.88–3.00) | 6.33× (6.22–6.45) | 2.30× (2.25–2.34) | 2.17× (2.12–2.21) | 0.92× (0.90–0.94) | 1.42× (1.39–1.44) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 0.092 s | 134 | 24.6 | 1.42× (1.41–1.43) | 3.01× (2.96–3.06) | 3.27× (3.24–3.31) | 3.23× (3.19–3.28) | 3.43× (3.38–3.48) | 3.67× (3.62–3.71) | 3.40× (3.36–3.45) | 1.11× (1.08–1.14) | 1.33× (1.31–1.34) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 0.059 s | 209 | 28.3 | 1.21× (1.15–1.26) | 2.57× (2.41–2.74) | 2.96× (2.79–3.15) | 3.21× (3.04–3.40) | 3.26× (3.12–3.40) | 5.61× (5.39–5.85) | 5.21× (4.99–5.43) | 1.70× (1.61–1.79) | 2.03× (1.94–2.12) | yes |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 0.047 s | 262 | 39.7 | 1.31× (1.26–1.35) | 2.07× (1.97–2.18) | 2.68× (2.55–2.83) | 2.96× (2.84–3.09) | 3.22× (3.10–3.34) | 7.05× (6.84–7.27) | 6.54× (6.34–6.75) | 2.13× (2.03–2.23) | 2.55× (2.47–2.63) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.34× (2.33–2.35) | 5.38× (5.36–5.40) | 5.55× (5.52–5.57) | 6.01× (5.99–6.03) | 5.41× (5.39–5.43) | 5.80× (5.77–5.83) | 5.42× (5.39–5.45) | 1.75× (1.74–1.77) | 2.02× (2.01–2.03) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.25× (2.22–2.27) | 5.18× (5.09–5.28) | 5.36× (5.29–5.43) | 5.88× (5.71–6.06) | 5.08× (5.01–5.15) | 5.33× (5.25–5.40) | 4.98× (4.92–5.04) | 1.61× (1.59–1.63) | 1.86× (1.83–1.88) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.25× (2.21–2.29) | 3.83× (3.75–3.91) | 4.13× (4.04–4.23) | 4.35× (4.26–4.45) | 3.89× (3.81–3.97) | 3.35× (3.27–3.44) | 3.13× (3.06–3.21) | 1.01× (0.99–1.04) | 1.17× (1.14–1.20) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.62–0.64) | 3.29× (3.23–3.36) | 4.06× (4.00–4.11) | 3.35× (3.30–3.39) | 5.70× (5.62–5.79) | 2.66× (2.63–2.70) | 2.50× (2.47–2.54) | 1.02× (1.00–1.03) | 1.09× (1.06–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 3.04× (2.97–3.11) | 5.34× (5.23–5.44) | 3.27× (3.21–3.34) | 5.48× (5.38–5.57) | 2.58× (2.52–2.63) | 2.42× (2.37–2.48) | 0.98× (0.96–1.01) | 1.05× (1.02–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.36× (3.29–3.42) | 5.38× (5.26–5.51) | 3.31× (3.26–3.36) | 5.59× (5.48–5.70) | 2.60× (2.56–2.65) | 2.45× (2.41–2.49) | 0.99× (0.97–1.01) | 1.06× (1.03–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.39× (2.36–2.42) | 2.61× (2.59–2.63) | 3.29× (3.26–3.32) | 3.43× (3.41–3.46) | 3.72× (3.68–3.76) | 2.26× (2.24–2.28) | 1.92× (1.91–1.94) | 1.85× (1.84–1.86) | 2.55× (2.53–2.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.08× (2.04–2.12) | 2.31× (2.27–2.36) | 2.85× (2.81–2.89) | 2.92× (2.88–2.96) | 3.12× (3.07–3.16) | 1.81× (1.78–1.84) | 1.54× (1.52–1.57) | 1.49× (1.46–1.51) | 2.05× (2.02–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.86–2.03) | 1.62× (1.56–1.68) | 2.10× (2.04–2.16) | 1.96× (1.87–2.05) | 1.97× (1.90–2.04) | 0.94× (0.91–0.98) | 0.80× (0.77–0.83) | 0.77× (0.75–0.80) | 1.07× (1.03–1.11) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.36) | 2.71× (2.67–2.74) | 2.46× (2.42–2.50) | 2.50× (2.48–2.52) | 2.88× (2.85–2.91) | 3.11× (3.08–3.13) | 3.12× (3.10–3.14) | 1.27× (1.26–1.28) | 1.39× (1.38–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.27–1.29) | 2.50× (2.47–2.53) | 2.20× (2.17–2.22) | 2.94× (2.92–2.97) | 2.60× (2.56–2.63) | 2.74× (2.71–2.78) | 2.75× (2.72–2.79) | 1.12× (1.10–1.14) | 1.23× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.42–1.51) | 2.30× (2.23–2.37) | 1.89× (1.85–1.93) | 2.61× (2.56–2.66) | 2.48× (2.44–2.52) | 2.02× (1.99–2.05) | 2.02× (1.99–2.06) | 0.82× (0.81–0.84) | 0.90× (0.89–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.43–1.46) | 3.03× (3.02–3.05) | 2.57× (2.53–2.61) | 2.67× (2.63–2.71) | 3.13× (3.04–3.22) | 3.74× (3.72–3.76) | 3.84× (3.81–3.87) | 1.45× (1.42–1.48) | 1.67× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.32–1.36) | 2.78× (2.74–2.81) | 2.29× (2.27–2.32) | 3.23× (3.18–3.27) | 2.76× (2.73–2.78) | 3.26× (3.23–3.30) | 3.35× (3.32–3.38) | 1.26× (1.23–1.30) | 1.46× (1.44–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.45× (1.33–1.58) | 2.46× (2.40–2.52) | 1.87× (1.82–1.91) | 2.70× (2.64–2.77) | 2.48× (2.40–2.56) | 2.22× (2.17–2.27) | 2.28× (2.22–2.34) | 0.86× (0.84–0.89) | 0.99× (0.97–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.70–0.70) | 2.68× (2.65–2.70) | 3.52× (3.50–3.54) | 2.80× (2.79–2.82) | 5.98× (5.92–6.03) | 2.24× (2.23–2.26) | 2.13× (2.11–2.15) | 0.97× (0.96–0.98) | 0.97× (0.94–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.62–0.63) | 2.48× (2.44–2.53) | 4.25× (4.19–4.31) | 2.74× (2.70–2.78) | 5.67× (5.58–5.76) | 2.11× (2.08–2.14) | 2.00× (1.97–2.03) | 0.91× (0.90–0.93) | 0.91× (0.88–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.74× (2.70–2.79) | 4.32× (4.20–4.45) | 2.80× (2.75–2.85) | 5.73× (5.63–5.84) | 2.15× (2.11–2.18) | 2.03× (2.00–2.07) | 0.93× (0.91–0.95) | 0.92× (0.89–0.96) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 1.41× (1.40–1.42) | 2.90× (2.86–2.94) | 3.17× (3.13–3.21) | 3.18× (3.14–3.23) | 3.38× (3.33–3.42) | 3.61× (3.57–3.65) | 3.35× (3.31–3.39) | 1.11× (1.08–1.14) | 1.28× (1.27–1.29) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 1.33× (1.31–1.34) | 2.63× (2.59–2.68) | 2.91× (2.84–2.97) | 3.16× (3.09–3.23) | 2.99× (2.96–3.02) | 3.13× (3.09–3.17) | 2.91× (2.87–2.95) | 0.96× (0.94–0.98) | 1.11× (1.10–1.12) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 1.48× (1.43–1.52) | 2.31× (2.26–2.35) | 2.59× (2.55–2.64) | 2.69× (2.64–2.74) | 2.66× (2.61–2.71) | 2.23× (2.21–2.26) | 2.07× (2.05–2.10) | 0.68× (0.67–0.70) | 0.79× (0.78–0.80) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.32× (0.30–0.34) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.17× (12.16–12.19) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.56× (0.51–0.62) | 0.81× (0.80–0.81) | 0.66× (0.66–0.67) | 6.40× (6.37–6.43) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.70× (0.59–0.84) | 0.71× (0.70–0.72) | 0.62× (0.60–0.63) | 4.10× (4.08–4.12) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.80–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.72–1.73) | 0.66× (0.66–0.66) | 8.14× (8.11–8.16) | 0.64× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.68× (0.64–0.72) | 1.64× (1.61–1.66) | 0.80× (0.79–0.81) | 5.92× (5.83–6.00) | 0.47× (0.46–0.47) | 0.49× (0.49–0.50) | 0.53× (0.53–0.54) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.62× (0.58–0.66) | 0.71× (0.65–0.76) | 1.14× (1.08–1.20) | 0.73× (0.70–0.77) | 3.34× (3.18–3.51) | 0.25× (0.24–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.39–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.74× (0.73–0.74) | 5.26× (5.25–5.26) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.44) | 0.92× (0.89–0.95) | 0.71× (0.68–0.75) | 3.17× (3.07–3.28) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.46× (0.45–0.47) | 0.87× (0.86–0.89) | 0.80× (0.78–0.83) | 2.53× (2.48–2.58) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.83–0.84) | 5.28× (5.27–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.45–0.48) | 0.37× (0.34–0.40) | 0.85× (0.82–0.87) | 0.70× (0.67–0.74) | 2.90× (2.80–3.00) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.47× (0.45–0.49) | 0.80× (0.79–0.81) | 0.82× (0.80–0.85) | 2.31× (2.27–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.09) | 0.77× (0.76–0.77) | 5.44× (5.44–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.40–5.43) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.78) | 5.42× (5.41–5.44) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 1 | 18 MB | 0.47× (0.46–0.47) | 0.41× (0.41–0.41) | 1.50× (1.50–1.50) | 0.67× (0.66–0.67) | 6.75× (6.73–6.77) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 2 | 31 MB | 0.52× (0.51–0.53) | 0.48× (0.44–0.52) | 1.27× (1.24–1.29) | 0.78× (0.75–0.81) | 3.95× (3.89–4.02) | 0.24× (0.23–0.24) | 0.25× (0.25–0.25) | 0.39× (0.38–0.39) | 0.19× (0.19–0.20) |
| `text` (2.15 MB → 12.4 MB, 5.8×) | 4 | 50 MB | 0.56× (0.54–0.57) | 0.53× (0.49–0.58) | 1.02× (0.99–1.04) | 0.80× (0.75–0.85) | 2.61× (2.56–2.66) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 322 | 3.11 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 561 | 3.38 | 1.09× (1.07–1.10) | 88% | 1.05× (1.03–1.07) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 695 | 5.40 | 1.73× (1.69–1.77) | 54% | 1.23× (1.22–1.24) | 1.66× (1.64–1.69) |

**Best:** mibz2 at 4 threads (695 MB/s); lbzip2 at 4 threads (188 MB/s); pbz2 at 4 threads (293 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.69, mibz2 4t 1.06, lbzip2 1t 0.09, lbzip2 2t 0.40, lbzip2 4t 0.50, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.92 | 0.02 | 0.09 | 0.301 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3239 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 11.5% |
| MTF + RLE2 | 1.58 | 16.3% |
| T-vector build | 2.11 | 21.7% |
| walk (the one each build uses) | 4.36 | 44.8% |
| of which: segment assembly | 0.16 | 1.6% |
| RLE1 | 0.29 | 3.0% |
| CRC | 0.26 | 2.6% |
| **total** | 9.73 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.673 | 1.300 | (in its total) | 1.93× (1.93–1.93) |  |
| walk | 1.266 | 2.663 | (in its total) | 2.10× (2.10–2.11) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.058 | (in its total) | 1.21× (1.20–1.23) |  |
| inverse BWT, total | 1.986 | 4.021 | 3.282 | 2.02× (2.02–2.03) | 1.65× (1.64–1.66) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h34m ± 11m07s |
| 4 threads | 43m42s ± 6m10s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m13s ± 4m46s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 33m13s ± 4m46s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.35 ns a byte of output at 4 threads (CPU, summed over them), 2.60 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,148,587 | 12,367,175 | \`be4cf6e3036d6c66…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3239.2 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
