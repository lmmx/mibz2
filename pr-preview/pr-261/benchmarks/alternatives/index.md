# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-261/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`4973a5a`](<https://github.com/lmmx/mibz2/commit/4973a5a74ba8d453139dda8843219238e665809d>) (2026-11-08).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42474468280>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.645 s | 325 | 8.0 | 2.34× (2.31–2.37) | 5.35× (5.31–5.38) | 5.47× (5.43–5.50) | 5.97× (5.92–6.02) | 5.34× (5.30–5.39) | 5.77× (5.72–5.81) | 5.40× (5.36–5.44) | 1.74× (1.72–1.75) | 2.02× (2.00–2.04) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.354 s | 592 | 8.7 | 2.40× (2.28–2.52) | 5.02× (4.67–5.40) | 5.68× (5.29–6.09) | 5.61× (5.26–5.99) | 5.05× (4.72–5.40) | 10.15× (9.50–10.85) | 9.50× (8.88–10.16) | 3.06× (2.86–3.27) | 3.55× (3.32–3.80) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.295 s | 710 | 13.6 | 2.46× (2.32–2.61) | 3.62× (3.54–3.70) | 4.51× (4.39–4.63) | 4.09× (4.02–4.17) | 4.09× (4.01–4.17) | 12.57× (12.35–12.79) | 11.77× (11.61–11.92) | 3.79× (3.74–3.84) | 4.40× (4.34–4.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 993 | 2.9 | 0.61× (0.60–0.62) | 3.76× (3.56–3.96) | 4.56× (4.46–4.66) | 3.63× (3.59–3.66) | 6.25× (6.20–6.29) | 2.88× (2.85–2.91) | 2.71× (2.68–2.74) | 1.02× (1.01–1.03) | 1.53× (1.52–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 968 | 3.0 | 0.57× (0.56–0.58) | 3.01× (2.96–3.05) | 5.94× (5.85–6.02) | 3.57× (3.53–3.62) | 6.10× (6.02–6.19) | 2.82× (2.79–2.86) | 2.65× (2.62–2.69) | 1.00× (0.98–1.02) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 964 | 3.0 | 0.56× (0.53–0.58) | 2.92× (2.78–3.07) | 5.79× (5.60–5.98) | 3.45× (3.29–3.62) | 5.90× (5.60–6.21) | 2.73× (2.60–2.87) | 2.56× (2.44–2.69) | 0.97× (0.91–1.02) | 1.45× (1.38–1.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 551 | 5.0 | 2.46× (2.43–2.49) | 2.85× (2.82–2.88) | 3.71× (3.58–3.85) | 3.60× (3.57–3.64) | 3.89× (3.87–3.91) | 2.35× (2.33–2.37) | 1.99× (1.97–2.00) | 1.89× (1.88–1.91) | 2.84× (2.81–2.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 860 | 6.1 | 2.20× (2.11–2.30) | 2.87× (2.65–3.10) | 4.20× (3.85–4.59) | 3.70× (3.13–4.38) | 4.65× (4.58–4.72) | 3.69× (3.65–3.73) | 3.11× (3.08–3.15) | 2.97× (2.94–3.00) | 4.45× (4.40–4.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 748 | 12.1 | 1.77× (1.68–1.86) | 1.72× (1.57–1.89) | 2.78× (2.56–3.02) | 2.09× (1.87–2.34) | 3.59× (3.32–3.88) | 3.08× (2.85–3.33) | 2.60× (2.41–2.81) | 2.48× (2.29–2.68) | 3.72× (3.43–4.02) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.128 s | 63 | 42.1 | 1.36× (1.34–1.37) | 2.74× (2.69–2.78) | 2.46× (2.42–2.50) | 2.54× (2.51–2.57) | 2.89× (2.86–2.92) | 3.16× (3.13–3.20) | 3.17× (3.14–3.20) | 1.26× (1.24–1.28) | 1.43× (1.41–1.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 101 | 47.4 | 1.25× (1.20–1.30) | 2.50× (2.39–2.62) | 2.40× (2.33–2.47) | 3.06× (2.92–3.20) | 2.91× (2.79–3.04) | 5.02× (4.80–5.25) | 5.04× (4.83–5.25) | 2.01× (1.91–2.11) | 2.27× (2.16–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 115 | 66.0 | 1.27× (1.23–1.31) | 2.07× (1.99–2.16) | 1.99× (1.88–2.11) | 2.48× (2.40–2.56) | 2.75× (2.66–2.83) | 5.84× (5.65–6.03) | 5.85× (5.67–6.04) | 2.33× (2.24–2.43) | 2.64× (2.54–2.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 39.1 | 1.45× (1.43–1.47) | 3.08× (3.05–3.10) | 2.56× (2.53–2.58) | 2.69× (2.67–2.70) | 3.08× (3.05–3.11) | 3.79× (3.77–3.81) | 3.88× (3.84–3.91) | 1.44× (1.43–1.45) | 1.71× (1.69–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 44.2 | 1.34× (1.31–1.37) | 2.76× (2.71–2.80) | 2.50× (2.43–2.56) | 3.50× (3.41–3.58) | 3.10× (3.02–3.18) | 6.21× (6.10–6.32) | 6.36× (6.24–6.48) | 2.36× (2.32–2.41) | 2.80× (2.74–2.85) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 64.7 | 1.35× (1.31–1.40) | 2.25× (2.19–2.30) | 2.15× (2.08–2.21) | 2.90× (2.81–2.99) | 2.73× (2.64–2.81) | 6.90× (6.75–7.06) | 7.07× (6.92–7.23) | 2.63× (2.57–2.68) | 3.11× (3.04–3.19) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 4.0 | 0.66× (0.66–0.67) | 3.04× (3.01–3.07) | 4.46× (4.42–4.50) | 3.14× (3.08–3.19) | 6.78× (6.72–6.85) | 2.48× (2.45–2.50) | 2.33× (2.31–2.35) | 0.98× (0.97–0.99) | 1.53× (1.51–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 704 | 4.3 | 0.56× (0.55–0.56) | 2.88× (2.85–2.91) | 4.96× (4.90–5.01) | 2.99× (2.96–3.02) | 6.41× (6.35–6.48) | 2.34× (2.32–2.37) | 2.21× (2.18–2.24) | 0.92× (0.91–0.94) | 1.45× (1.43–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 699 | 4.2 | 0.56× (0.56–0.57) | 2.89× (2.86–2.92) | 4.95× (4.89–5.00) | 2.98× (2.95–3.01) | 6.39× (6.30–6.48) | 2.34× (2.31–2.37) | 2.20× (2.18–2.23) | 0.92× (0.91–0.94) | 1.45× (1.43–1.46) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 0.089 s | 142 | 18.6 | 1.45× (1.45–1.46) | 3.09× (3.08–3.10) | 3.29× (3.28–3.30) | 3.31× (3.30–3.33) | 3.49× (3.48–3.51) | 3.76× (3.75–3.78) | 3.53× (3.51–3.55) | 1.13× (1.11–1.14) | 1.37× (1.36–1.37) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 0.056 s | 225 | 21.7 | 1.22× (1.04–1.44) | 2.62× (2.16–3.18) | 2.94× (2.55–3.40) | 3.07× (2.69–3.50) | 3.16× (2.80–3.57) | 5.39× (4.78–6.08) | 5.06× (4.48–5.72) | 1.61× (1.43–1.83) | 1.96× (1.74–2.21) | yes |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 0.046 s | 276 | 30.3 | 1.34× (1.29–1.40) | 2.11× (1.99–2.24) | 2.84× (2.70–3.00) | 2.95× (2.86–3.04) | 3.35× (3.25–3.45) | 7.30× (7.11–7.50) | 6.85× (6.67–7.04) | 2.19× (2.12–2.25) | 2.65× (2.58–2.73) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.34× (2.31–2.36) | 5.33× (5.29–5.36) | 5.45× (5.41–5.48) | 5.95× (5.91–6.00) | 5.33× (5.28–5.38) | 5.75× (5.71–5.80) | 5.38× (5.35–5.42) | 1.74× (1.72–1.75) | 2.01× (1.99–2.03) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.24× (2.23–2.26) | 5.05× (5.00–5.10) | 5.29× (5.25–5.33) | 5.81× (5.77–5.86) | 4.99× (4.96–5.02) | 5.32× (5.27–5.36) | 4.98× (4.94–5.01) | 1.60× (1.59–1.62) | 1.86× (1.84–1.87) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.28× (2.22–2.33) | 3.80× (3.72–3.89) | 4.09× (4.00–4.19) | 4.31× (4.21–4.40) | 3.78× (3.70–3.86) | 3.39× (3.32–3.47) | 3.18× (3.11–3.24) | 1.02× (1.00–1.04) | 1.19× (1.16–1.21) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.64× (0.63–0.65) | 3.31× (3.27–3.36) | 4.15× (4.08–4.23) | 3.39× (3.36–3.42) | 5.78× (5.73–5.82) | 2.71× (2.69–2.73) | 2.55× (2.53–2.58) | 1.02× (1.01–1.03) | 1.10× (1.06–1.15) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.09× (3.05–3.13) | 5.35× (5.29–5.40) | 3.31× (3.28–3.35) | 5.57× (5.50–5.64) | 2.62× (2.59–2.64) | 2.46× (2.44–2.49) | 0.98× (0.96–1.00) | 1.07× (1.02–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.56–0.62) | 3.32× (3.14–3.52) | 5.28× (5.02–5.56) | 3.26× (3.08–3.44) | 5.44× (5.14–5.77) | 2.57× (2.43–2.72) | 2.42× (2.29–2.56) | 0.97× (0.91–1.02) | 1.05× (0.97–1.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.38× (2.35–2.41) | 2.63× (2.60–2.66) | 3.33× (3.21–3.45) | 3.47× (3.43–3.50) | 3.74× (3.72–3.76) | 2.28× (2.26–2.30) | 1.93× (1.92–1.95) | 1.85× (1.83–1.87) | 2.55× (2.53–2.58) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.11× (2.08–2.14) | 2.39× (2.34–2.45) | 2.89× (2.84–2.93) | 3.01× (2.98–3.04) | 3.18× (3.13–3.22) | 1.87× (1.84–1.89) | 1.58× (1.56–1.60) | 1.51× (1.49–1.53) | 2.09× (2.06–2.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.99× (1.91–2.06) | 1.68× (1.60–1.76) | 2.12× (2.03–2.21) | 1.97× (1.88–2.06) | 2.02× (1.94–2.09) | 0.97× (0.93–1.01) | 0.82× (0.79–0.85) | 0.79× (0.75–0.82) | 1.08× (1.04–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.35× (1.34–1.37) | 2.68× (2.65–2.71) | 2.41× (2.38–2.44) | 2.52× (2.49–2.55) | 2.86× (2.83–2.89) | 3.13× (3.10–3.17) | 3.14× (3.11–3.17) | 1.26× (1.24–1.28) | 1.39× (1.38–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.29) | 2.50× (2.47–2.54) | 2.21× (2.19–2.23) | 2.98× (2.95–3.00) | 2.60× (2.58–2.63) | 2.78× (2.76–2.81) | 2.79× (2.77–2.82) | 1.12× (1.10–1.14) | 1.24× (1.22–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.41–1.50) | 2.27× (2.22–2.32) | 1.84× (1.80–1.89) | 2.55× (2.50–2.59) | 2.41× (2.35–2.46) | 2.01× (1.97–2.05) | 2.02× (1.98–2.06) | 0.81× (0.79–0.83) | 0.89× (0.87–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.46) | 3.01× (2.99–3.03) | 2.52× (2.50–2.54) | 2.66× (2.65–2.68) | 3.05× (3.03–3.08) | 3.75× (3.73–3.77) | 3.84× (3.80–3.87) | 1.44× (1.43–1.44) | 1.66× (1.65–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.35× (1.34–1.35) | 2.78× (2.76–2.79) | 2.31× (2.30–2.33) | 3.27× (3.23–3.31) | 2.78× (2.72–2.84) | 3.31× (3.28–3.34) | 3.39× (3.36–3.42) | 1.27× (1.26–1.28) | 1.47× (1.45–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.52–1.56) | 2.47× (2.42–2.53) | 1.89× (1.86–1.92) | 2.71× (2.67–2.76) | 2.44× (2.40–2.49) | 2.27× (2.23–2.31) | 2.32× (2.29–2.36) | 0.87× (0.85–0.89) | 1.01× (0.98–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.71× (2.69–2.74) | 3.55× (3.52–3.59) | 2.86× (2.81–2.91) | 6.05× (5.98–6.11) | 2.28× (2.26–2.31) | 2.16× (2.14–2.19) | 0.98× (0.97–0.99) | 0.98× (0.96–1.00) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.63) | 2.52× (2.49–2.55) | 4.26× (4.21–4.31) | 2.77× (2.74–2.80) | 5.68× (5.62–5.73) | 2.14× (2.12–2.16) | 2.02× (2.00–2.05) | 0.92× (0.91–0.93) | 0.92× (0.90–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.61–0.62) | 2.78× (2.74–2.81) | 4.29× (4.24–4.34) | 2.83× (2.80–2.86) | 5.79× (5.70–5.87) | 2.19× (2.16–2.21) | 2.07× (2.04–2.10) | 0.94× (0.92–0.96) | 0.94× (0.90–0.97) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 1.44× (1.44–1.45) | 2.96× (2.95–2.97) | 3.20× (3.19–3.22) | 3.26× (3.25–3.28) | 3.44× (3.42–3.45) | 3.70× (3.68–3.72) | 3.48× (3.46–3.50) | 1.12× (1.11–1.14) | 1.31× (1.30–1.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 1.32× (1.30–1.34) | 2.62× (2.59–2.66) | 2.90× (2.86–2.95) | 3.17× (3.13–3.21) | 3.01× (2.98–3.05) | 3.17× (3.13–3.22) | 2.98× (2.94–3.02) | 0.96× (0.95–0.98) | 1.13× (1.11–1.14) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 1.48× (1.44–1.53) | 2.26× (2.22–2.30) | 2.51× (2.47–2.55) | 2.69× (2.62–2.76) | 2.58× (2.54–2.63) | 2.26× (2.22–2.29) | 2.12× (2.08–2.16) | 0.69× (0.67–0.70) | 0.80× (0.79–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.32–0.33) | 0.33× (0.32–0.35) | 1.08× (1.08–1.08) | 0.86× (0.85–0.86) | 12.17× (12.15–12.19) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.46) | 0.24× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.38–0.39) | 0.59× (0.51–0.70) | 0.81× (0.79–0.82) | 0.67× (0.65–0.68) | 6.41× (6.35–6.47) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.69× (0.62–0.78) | 0.71× (0.70–0.72) | 0.61× (0.60–0.63) | 4.12× (4.09–4.15) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.17× (0.16–0.18) | 0.82× (0.82–0.83) | 0.81× (0.81–0.81) | 4.32× (4.32–4.32) | 0.25× (0.25–0.25) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.79–1.80) | 0.84× (0.84–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.79× (1.79–1.80) | 0.84× (0.84–0.84) | 4.42× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.50× (0.50–0.51) | 0.45× (0.45–0.45) | 1.73× (1.73–1.73) | 0.66× (0.66–0.66) | 8.13× (8.11–8.15) | 0.64× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.74–0.74) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.67) | 0.74× (0.67–0.82) | 1.64× (1.62–1.67) | 0.81× (0.80–0.83) | 5.94× (5.86–6.02) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.60–0.66) | 0.71× (0.66–0.77) | 1.15× (1.10–1.21) | 0.73× (0.70–0.75) | 3.36× (3.21–3.52) | 0.26× (0.24–0.27) | 0.27× (0.26–0.28) | 0.29× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.20) | 0.74× (0.73–0.74) | 5.25× (5.24–5.26) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.53) | 0.40× (0.36–0.44) | 0.92× (0.89–0.96) | 0.69× (0.66–0.72) | 3.17× (3.06–3.27) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.82× (0.80–0.84) | 2.54× (2.49–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.19–1.20) | 0.84× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.51–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.46–0.49) | 0.36× (0.33–0.40) | 0.86× (0.83–0.90) | 0.71× (0.68–0.75) | 2.95× (2.85–3.06) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.60) | 0.48× (0.45–0.51) | 0.81× (0.79–0.82) | 0.79× (0.76–0.82) | 2.33× (2.29–2.36) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.21× (0.21–0.21) | 1.09× (1.09–1.10) | 0.77× (0.77–0.77) | 5.45× (5.44–5.45) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.77) | 5.39× (5.37–5.42) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.65) | 0.77× (0.77–0.78) | 5.42× (5.39–5.44) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 1 | 18 MB | 0.46× (0.46–0.46) | 0.40× (0.40–0.41) | 1.50× (1.50–1.50) | 0.66× (0.66–0.67) | 6.75× (6.74–6.76) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 2 | 34 MB | 0.47× (0.45–0.48) | 0.38× (0.36–0.41) | 1.15× (1.10–1.21) | 0.71× (0.69–0.74) | 3.57× (3.45–3.69) | 0.21× (0.21–0.22) | 0.23× (0.22–0.23) | 0.35× (0.34–0.36) | 0.17× (0.17–0.18) |
| `text` (2.07 MB → 12.7 MB, 6.1×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.47× (0.44–0.49) | 0.96× (0.94–0.98) | 0.78× (0.76–0.79) | 2.44× (2.40–2.49) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 325 | 3.08 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 592 | 3.33 | 1.08× (1.07–1.09) | 88% | 1.03× (1.02–1.03) | 1.04× (1.04–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 710 | 5.23 | 1.70× (1.66–1.74) | 54% | 1.21× (1.20–1.22) | 1.65× (1.63–1.68) |

**Best:** mibz2 at 4 threads (710 MB/s); lbzip2 at 4 threads (197 MB/s); pbz2 at 4 threads (297 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.67, mibz2 4t 1.06, lbzip2 1t 0.09, lbzip2 2t 0.37, lbzip2 4t 0.53, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 7%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 4%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.90 | 0.02 | 0.07 | 0.295 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2593 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.90 | 11.8% |
| MTF + RLE2 | 1.27 | 16.7% |
| T-vector build | 1.69 | 22.3% |
| walk (the one each build uses) | 3.29 | 43.4% |
| of which: segment assembly | 0.12 | 1.6% |
| RLE1 | 0.23 | 3.1% |
| CRC | 0.20 | 2.7% |
| **total** | 7.58 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.670 | 1.300 | (in its total) | 1.94× (1.94–1.94) |  |
| walk | 1.231 | 2.626 | (in its total) | 2.13× (2.12–2.13) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.046 | 0.055 | (in its total) | 1.19× (1.16–1.21) |  |
| inverse BWT, total | 1.947 | 3.981 | 3.277 | 2.04× (2.03–2.05) | 1.68× (1.68–1.69) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h33m ± 11m01s |
| 4 threads | 43m01s ± 6m03s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m22s ± 4m39s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 32m22s ± 4m39s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.24 ns a byte of output at 4 threads (CPU, summed over them), 2.57 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,073,075 | 12,677,537 | \`e4ea56361d34c699…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2592.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
