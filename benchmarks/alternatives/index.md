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

The latest record: [`0e6ab25`](<https://github.com/lmmx/mibz2/commit/0e6ab2544cadca7615832fbf9511dc9702f2e885>) (2026-10-29).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40859536470>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.768 s | 273 | 11.9 | 1.98× (1.97–1.98) | 4.44× (4.42–4.47) | 4.64× (4.62–4.66) | 4.98× (4.95–5.01) | 4.50× (4.45–4.55) | 4.83× (4.81–4.86) | 4.49× (4.47–4.52) | 1.45× (1.44–1.45) | 1.70× (1.69–1.72) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.488 s | 429 | 13.0 | 1.74× (1.67–1.82) | 3.61× (3.52–3.71) | 4.23× (4.15–4.33) | 4.18× (4.06–4.30) | 3.80× (3.69–3.92) | 7.51× (7.30–7.72) | 6.98× (6.78–7.18) | 2.25× (2.18–2.31) | 2.65× (2.56–2.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.400 s | 524 | 20.5 | 1.74× (1.71–1.76) | 2.68× (2.66–2.70) | 3.31× (3.26–3.35) | 3.02× (2.98–3.05) | 3.07× (3.03–3.11) | 9.27× (9.14–9.39) | 8.61× (8.50–8.73) | 2.77× (2.74–2.81) | 3.27× (3.24–3.30) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 935 | 3.8 | 0.58× (0.57–0.59) | 3.78× (3.65–3.91) | 4.74× (4.70–4.79) | 3.43× (3.39–3.46) | 6.46× (6.39–6.53) | 2.72× (2.69–2.75) | 2.55× (2.52–2.58) | 1.00× (0.99–1.01) | 1.45× (1.43–1.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 937 | 3.8 | 0.58× (0.57–0.59) | 2.90× (2.85–2.94) | 6.21× (6.12–6.30) | 3.47× (3.42–3.52) | 6.44× (6.35–6.53) | 2.73× (2.70–2.77) | 2.56× (2.53–2.60) | 1.01× (0.99–1.02) | 1.45× (1.43–1.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 930 | 3.8 | 0.57× (0.56–0.58) | 2.88× (2.84–2.91) | 6.15× (6.08–6.22) | 3.43× (3.40–3.47) | 6.43× (6.38–6.49) | 2.70× (2.67–2.73) | 2.53× (2.50–2.56) | 0.99× (0.98–1.01) | 1.44× (1.42–1.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.044 s | 443 | 7.6 | 2.00× (1.98–2.02) | 2.30× (2.28–2.33) | 2.74× (2.71–2.77) | 2.90× (2.86–2.93) | 3.42× (3.38–3.46) | 1.90× (1.88–1.92) | 1.61× (1.59–1.62) | 1.57× (1.55–1.59) | 2.29× (2.27–2.32) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.028 s | 713 | 9.1 | 1.73× (1.58–1.90) | 2.40× (2.18–2.64) | 3.21× (2.94–3.51) | 2.74× (2.54–2.97) | 4.15× (3.81–4.53) | 2.95× (2.69–3.22) | 2.49× (2.28–2.72) | 2.43× (2.22–2.66) | 3.56× (3.25–3.89) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.030 s | 646 | 17.7 | 1.52× (1.49–1.55) | 1.77× (1.70–1.84) | 2.51× (2.41–2.61) | 1.92× (1.86–2.00) | 3.62× (3.57–3.66) | 2.77× (2.75–2.79) | 2.34× (2.31–2.37) | 2.28× (2.26–2.30) | 3.34× (3.31–3.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.152 s | 53 | 62.4 | 1.17× (1.17–1.18) | 2.35× (2.35–2.36) | 2.12× (2.11–2.14) | 2.10× (2.09–2.12) | 2.52× (2.51–2.53) | 2.67× (2.66–2.68) | 2.66× (2.65–2.66) | 1.05× (1.05–1.06) | 1.20× (1.20–1.21) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.093 s | 86 | 69.1 | 1.09× (1.04–1.13) | 2.18× (2.10–2.26) | 2.05× (1.98–2.13) | 2.61× (2.50–2.72) | 2.65× (2.55–2.75) | 4.30× (4.14–4.47) | 4.28× (4.12–4.45) | 1.70× (1.63–1.76) | 1.94× (1.87–2.01) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 107 | 92.4 | 1.22× (1.18–1.27) | 1.92× (1.87–1.97) | 1.87× (1.80–1.94) | 2.34× (2.26–2.42) | 2.72× (2.65–2.78) | 5.38× (5.26–5.51) | 5.35× (5.23–5.47) | 2.12× (2.07–2.17) | 2.42× (2.37–2.48) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.201 s | 48 | 68.6 | 1.03× (0.96–1.11) | 2.20× (2.05–2.36) | 1.80× (1.69–1.92) | 1.80× (1.67–1.93) | 2.24× (2.04–2.45) | 2.60× (2.42–2.79) | 2.62× (2.44–2.81) | 0.96× (0.90–1.04) | 1.12× (1.04–1.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.119 s | 81 | 74.8 | 1.03× (0.96–1.11) | 2.12× (2.07–2.18) | 1.83× (1.77–1.88) | 2.58× (2.50–2.65) | 2.37× (2.31–2.43) | 4.54× (4.42–4.66) | 4.58× (4.46–4.71) | 1.69× (1.64–1.74) | 1.95× (1.90–2.01) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.095 s | 102 | 100.9 | 1.18× (1.16–1.19) | 1.89× (1.84–1.93) | 1.82× (1.77–1.88) | 2.47× (2.43–2.50) | 2.40× (2.37–2.43) | 5.78× (5.71–5.86) | 5.84× (5.77–5.91) | 2.15× (2.12–2.18) | 2.49× (2.46–2.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 687 | 5.3 | 0.63× (0.63–0.64) | 2.90× (2.77–3.05) | 4.19× (4.15–4.23) | 2.92× (2.89–2.95) | 7.16× (7.11–7.22) | 2.32× (2.30–2.35) | 2.18× (2.15–2.20) | 0.97× (0.95–0.98) | 1.43× (1.42–1.45) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 671 | 5.6 | 0.56× (0.56–0.57) | 2.77× (2.74–2.80) | 4.80× (4.74–4.85) | 2.89× (2.86–2.92) | 7.03× (6.92–7.15) | 2.27× (2.24–2.29) | 2.12× (2.10–2.15) | 0.94× (0.93–0.96) | 1.40× (1.38–1.41) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 669 | 5.5 | 0.56× (0.55–0.57) | 2.76× (2.73–2.79) | 4.75× (4.69–4.81) | 2.88× (2.84–2.92) | 6.97× (6.84–7.10) | 2.25× (2.22–2.27) | 2.11× (2.08–2.13) | 0.93× (0.92–0.95) | 1.39× (1.37–1.40) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 0.086 s | 142 | 23.3 | 1.38× (1.37–1.38) | 2.84× (2.82–2.86) | 3.17× (3.14–3.19) | 3.08× (3.06–3.10) | 3.42× (3.39–3.46) | 3.31× (3.29–3.34) | 3.10× (3.07–3.14) | 1.01× (1.00–1.02) | 1.30× (1.29–1.31) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 0.053 s | 230 | 27.2 | 1.18× (1.13–1.25) | 2.46× (2.33–2.59) | 3.03× (2.86–3.21) | 3.33× (3.08–3.60) | 3.57× (3.32–3.83) | 5.24× (4.98–5.51) | 4.91× (4.67–5.15) | 1.60× (1.52–1.68) | 2.05× (1.95–2.16) | yes |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 0.047 s | 262 | 40.3 | 1.19× (1.15–1.23) | 1.98× (1.87–2.10) | 2.49× (2.38–2.60) | 2.71× (2.62–2.79) | 3.23× (3.14–3.32) | 6.10× (5.92–6.29) | 5.71× (5.55–5.87) | 1.86× (1.80–1.92) | 2.39× (2.32–2.46) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.97× (1.96–1.98) | 4.43× (4.40–4.45) | 4.62× (4.60–4.64) | 4.97× (4.94–5.00) | 4.49× (4.45–4.54) | 4.82× (4.80–4.85) | 4.48× (4.46–4.51) | 1.45× (1.44–1.45) | 1.70× (1.68–1.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 1.88× (1.87–1.89) | 4.18× (4.15–4.21) | 4.45× (4.43–4.46) | 4.85× (4.83–4.87) | 4.18× (4.17–4.20) | 4.42× (4.40–4.44) | 4.11× (4.09–4.13) | 1.32× (1.32–1.33) | 1.56× (1.54–1.57) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.90× (1.88–1.92) | 3.16× (3.13–3.20) | 3.38× (3.34–3.43) | 3.58× (3.55–3.61) | 3.16× (3.13–3.20) | 2.80× (2.78–2.82) | 2.61× (2.59–2.63) | 0.84× (0.83–0.85) | 0.99× (0.98–1.00) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 3.20× (3.17–3.23) | 4.09× (4.05–4.13) | 3.23× (3.20–3.27) | 6.02× (5.97–6.07) | 2.58× (2.56–2.61) | 2.42× (2.40–2.45) | 1.00× (0.99–1.01) | 1.05× (1.03–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.03× (2.99–3.08) | 5.35× (5.30–5.41) | 3.23× (3.20–3.27) | 5.90× (5.83–5.97) | 2.55× (2.51–2.58) | 2.39× (2.36–2.42) | 0.99× (0.97–1.00) | 1.04× (1.00–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.62) | 3.36× (3.32–3.41) | 5.43× (5.36–5.49) | 3.28× (3.25–3.32) | 6.01× (5.96–6.06) | 2.57× (2.55–2.60) | 2.42× (2.38–2.45) | 1.00× (0.98–1.01) | 1.05× (1.02–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.96× (1.94–1.98) | 2.22× (2.20–2.25) | 2.64× (2.61–2.67) | 2.82× (2.79–2.85) | 3.33× (3.29–3.36) | 1.87× (1.85–1.88) | 1.58× (1.57–1.60) | 1.54× (1.53–1.56) | 2.12× (2.08–2.16) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 1.78× (1.76–1.80) | 2.03× (2.00–2.07) | 2.49× (2.45–2.53) | 2.49× (2.45–2.53) | 2.88× (2.83–2.93) | 1.56× (1.54–1.58) | 1.32× (1.30–1.34) | 1.29× (1.27–1.31) | 1.77× (1.73–1.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.62× (1.58–1.67) | 1.48× (1.45–1.52) | 1.78× (1.75–1.82) | 1.70× (1.67–1.73) | 1.78× (1.75–1.82) | 0.80× (0.78–0.82) | 0.68× (0.66–0.70) | 0.66× (0.65–0.68) | 0.91× (0.88–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.17× (1.17–1.18) | 2.31× (2.30–2.32) | 2.06× (2.05–2.07) | 2.09× (2.07–2.11) | 2.50× (2.49–2.51) | 2.65× (2.64–2.66) | 2.64× (2.63–2.65) | 1.05× (1.05–1.06) | 1.14× (1.14–1.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.10× (1.06–1.14) | 2.20× (2.12–2.28) | 1.93× (1.86–1.99) | 2.52× (2.43–2.61) | 2.31× (2.23–2.38) | 2.36× (2.28–2.45) | 2.35× (2.27–2.43) | 0.94× (0.91–0.97) | 1.02× (0.98–1.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.34× (1.30–1.39) | 2.04× (2.01–2.07) | 1.70× (1.67–1.72) | 2.29× (2.24–2.35) | 2.23× (2.21–2.26) | 1.80× (1.78–1.82) | 1.79× (1.77–1.81) | 0.71× (0.70–0.72) | 0.77× (0.76–0.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.03× (0.96–1.11) | 2.16× (2.01–2.32) | 1.76× (1.65–1.88) | 1.79× (1.67–1.92) | 2.23× (2.04–2.43) | 2.58× (2.41–2.77) | 2.61× (2.43–2.79) | 0.96× (0.90–1.04) | 1.10× (1.02–1.18) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.03× (1.02–1.03) | 2.15× (2.13–2.16) | 1.75× (1.74–1.76) | 2.47× (2.43–2.52) | 2.12× (2.11–2.14) | 2.45× (2.44–2.47) | 2.48× (2.47–2.49) | 0.92× (0.91–0.92) | 1.04× (1.04–1.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.29× (1.28–1.30) | 2.03× (1.99–2.06) | 1.54× (1.52–1.56) | 2.25× (2.21–2.29) | 2.03× (2.00–2.06) | 1.83× (1.81–1.85) | 1.84× (1.83–1.86) | 0.68× (0.68–0.69) | 0.78× (0.77–0.78) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.67–0.69) | 2.60× (2.56–2.65) | 3.52× (3.47–3.56) | 2.72× (2.68–2.75) | 6.48× (6.43–6.54) | 2.18× (2.15–2.21) | 2.05× (2.02–2.08) | 0.97× (0.96–0.99) | 0.98× (0.95–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.62–0.63) | 2.46× (2.44–2.48) | 4.21× (4.16–4.25) | 2.67× (2.64–2.70) | 6.21× (6.13–6.30) | 2.07× (2.05–2.08) | 1.94× (1.93–1.96) | 0.92× (0.91–0.93) | 0.93× (0.90–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.73× (2.69–2.76) | 4.29× (4.24–4.35) | 2.76× (2.72–2.79) | 6.34× (6.23–6.46) | 2.11× (2.09–2.13) | 1.98× (1.96–2.01) | 0.94× (0.93–0.96) | 0.95× (0.92–0.98) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 1.37× (1.36–1.38) | 2.77× (2.75–2.79) | 3.09× (3.06–3.11) | 3.04× (3.01–3.06) | 3.37× (3.34–3.41) | 3.27× (3.24–3.29) | 3.06× (3.03–3.09) | 1.01× (1.00–1.02) | 1.21× (1.20–1.22) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 1.25× (1.24–1.26) | 2.49× (2.45–2.54) | 2.81× (2.80–2.83) | 2.92× (2.89–2.94) | 2.98× (2.96–3.01) | 2.80× (2.78–2.83) | 2.62× (2.60–2.65) | 0.87× (0.86–0.87) | 1.04× (1.03–1.05) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 1.38× (1.33–1.44) | 2.04× (1.99–2.08) | 2.31× (2.27–2.35) | 2.35× (2.31–2.38) | 2.40× (2.37–2.43) | 1.90× (1.87–1.92) | 1.78× (1.75–1.80) | 0.59× (0.58–0.60) | 0.70× (0.69–0.71) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.30× (0.28–0.32) | 1.17× (1.17–1.17) | 0.84× (0.84–0.84) | 11.87× (11.85–11.89) | 0.28× (0.28–0.28) | 0.29× (0.29–0.30) | 0.45× (0.45–0.45) | 0.23× (0.23–0.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.58× (0.52–0.66) | 0.86× (0.85–0.87) | 0.67× (0.66–0.68) | 6.36× (6.31–6.41) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.81× (0.70–0.94) | 0.78× (0.78–0.79) | 0.64× (0.63–0.65) | 4.29× (4.28–4.31) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.16× (0.16–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.26× (0.26–0.27) | 0.17× (0.16–0.18) | 0.82× (0.81–0.82) | 0.80× (0.80–0.80) | 4.29× (4.28–4.30) | 0.25× (0.25–0.25) | 0.26× (0.26–0.27) | 0.90× (0.90–0.91) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.19× (0.19–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.83) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.92× (0.92–0.93) | 0.10× (0.10–0.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.18–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.83× (0.83–0.84) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.93× (0.92–0.93) | 0.10× (0.10–0.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.49× (0.48–0.49) | 0.43× (0.42–0.43) | 1.64× (1.64–1.65) | 0.64× (0.63–0.64) | 7.81× (7.79–7.82) | 0.62× (0.61–0.62) | 0.65× (0.65–0.65) | 0.71× (0.71–0.71) | 0.35× (0.35–0.35) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.62× (0.62–0.62) | 0.67× (0.64–0.72) | 1.53× (1.53–1.54) | 0.87× (0.86–0.87) | 5.56× (5.55–5.57) | 0.44× (0.43–0.44) | 0.46× (0.46–0.46) | 0.50× (0.50–0.51) | 0.25× (0.25–0.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.59× (0.59–0.60) | 0.66× (0.61–0.70) | 1.11× (1.09–1.13) | 0.86× (0.85–0.86) | 3.22× (3.19–3.26) | 0.24× (0.24–0.25) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.14× (0.14–0.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.38× (0.38–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.71× (0.71–0.71) | 5.08× (5.07–5.09) | 0.31× (0.31–0.31) | 0.33× (0.33–0.33) | 0.51× (0.50–0.51) | 0.25× (0.25–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 36 MB | 0.51× (0.50–0.52) | 0.42× (0.39–0.45) | 0.94× (0.94–0.95) | 0.75× (0.73–0.77) | 3.29× (3.26–3.31) | 0.20× (0.20–0.20) | 0.21× (0.21–0.21) | 0.32× (0.32–0.33) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.57–0.61) | 0.42× (0.41–0.43) | 0.80× (0.79–0.82) | 0.76× (0.74–0.78) | 2.36× (2.31–2.40) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.38–0.39) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.81× (0.81–0.81) | 5.11× (5.10–5.12) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) | 0.50× (0.50–0.50) | 0.25× (0.25–0.25) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 40 MB | 0.48× (0.47–0.49) | 0.39× (0.37–0.41) | 0.87× (0.86–0.88) | 0.70× (0.68–0.72) | 3.01× (2.99–3.03) | 0.18× (0.18–0.18) | 0.19× (0.19–0.19) | 0.29× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.55× (0.54–0.56) | 0.48× (0.47–0.50) | 0.74× (0.73–0.76) | 0.75× (0.72–0.78) | 2.15× (2.11–2.20) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.09× (1.09–1.09) | 0.76× (0.76–0.76) | 5.43× (5.42–5.44) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.63× (1.62–1.63) | 0.77× (0.77–0.77) | 5.40× (5.39–5.41) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) | 0.91× (0.91–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.20× (0.20–0.20) | 1.63× (1.62–1.63) | 0.77× (0.76–0.77) | 5.41× (5.40–5.42) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) | 0.91× (0.91–0.92) | 0.13× (0.12–0.13) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.46× (1.46–1.47) | 0.63× (0.62–0.63) | 6.58× (6.56–6.60) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.64× (0.64–0.64) | 0.31× (0.31–0.32) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 2 | 31 MB | 0.52× (0.51–0.53) | 0.46× (0.41–0.53) | 1.28× (1.25–1.30) | 0.78× (0.76–0.80) | 3.99× (3.89–4.10) | 0.24× (0.23–0.24) | 0.25× (0.24–0.26) | 0.39× (0.38–0.40) | 0.19× (0.19–0.19) |
| `text` (1.48 MB → 12.3 MB, 8.3×) | 4 | 49 MB | 0.53× (0.52–0.55) | 0.53× (0.49–0.57) | 1.03× (1.01–1.05) | 0.81× (0.79–0.83) | 2.62× (2.58–2.67) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 273 | 3.67 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 429 | 4.01 | 1.09× (1.09–1.10) | 78% | 1.03× (1.03–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 524 | 6.33 | 1.72× (1.71–1.73) | 48% | 1.23× (1.22–1.24) | 1.66× (1.64–1.67) |

**Best:** mibz2 at 4 threads (524 MB/s); lbzip2 at 4 threads (195 MB/s); pbz2 at 4 threads (300 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.71, mibz2 4t 0.94, lbzip2 1t 0.09, lbzip2 2t 0.31, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.37. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 7%, mibz2 4t 6%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.11 | 0.02 | 0.08 | 0.400 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.22 | 10.8% |
| MTF + RLE2 | 2.24 | 19.8% |
| T-vector build | 2.25 | 19.9% |
| walk (the one each build uses) | 5.08 | 44.8% |
| of which: segment assembly | 0.18 | 1.6% |
| RLE1 | 0.28 | 2.5% |
| CRC | 0.26 | 2.3% |
| **total** | 11.33 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.693 | 1.303 | (in its total) | 1.88× (1.88–1.88) |  |
| walk | 1.532 | 2.614 | (in its total) | 1.70× (1.69–1.71) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.053 | 0.053 | (in its total) | 1.01× (1.00–1.02) |  |
| inverse BWT, total | 2.278 | 3.970 | 3.325 | 1.74× (1.73–1.75) | 1.46× (1.45–1.46) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h51m ± 13m23s |
| 4 threads | 58m25s ± 8m13s |
| 4 threads, the sample's overlap gap as a fixed cost | 40m25s ± 5m41s |
| the scanner alone (its serial part) | 16m35s ± 32.2s |
| the workers alone, over 4 | 40m25s ± 5m41s |

At 4 threads the workers bound it. Per byte: the scanner 9.6 ns a byte of bz2, the workers 5.30 ns a byte of output at 4 threads (CPU, summed over them), 3.13 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,478,819 | 12,273,030 | \`80f51ade7d6ce928…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4
