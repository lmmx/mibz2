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

The latest record: [`e8e7831`](<https://github.com/lmmx/mibz2/commit/e8e7831ab189d8ecc274f5073ca12acd8c970d2f>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42792751113>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.611 s | 343 | 7.2 | 2.51× (2.50–2.51) | 5.63× (5.61–5.66) | 5.81× (5.78–5.85) | 6.31× (6.28–6.33) | 5.64× (5.59–5.70) | 6.18× (6.14–6.22) | 5.68× (5.63–5.73) | 1.84× (1.83–1.85) | 2.16× (2.14–2.17) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.336 s | 624 | 7.8 | 2.53× (2.43–2.64) | 5.30× (5.14–5.46) | 6.13× (5.94–6.33) | 6.13× (5.88–6.39) | 5.40× (5.22–5.58) | 10.98× (10.63–11.35) | 10.10× (9.80–10.41) | 3.26× (3.17–3.37) | 3.83× (3.71–3.96) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.292 s | 717 | 12.9 | 2.41× (2.37–2.45) | 3.68× (3.63–3.73) | 4.58× (4.52–4.63) | 4.20× (4.14–4.26) | 4.14× (4.08–4.21) | 12.87× (12.74–13.01) | 11.83× (11.69–11.98) | 3.83× (3.78–3.87) | 4.49× (4.44–4.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 971 | 2.8 | 0.60× (0.59–0.60) | 3.63× (3.46–3.80) | 4.50× (4.35–4.65) | 3.55× (3.51–3.60) | 6.07× (5.99–6.15) | 2.96× (2.93–3.00) | 2.65× (2.61–2.68) | 1.00× (0.98–1.02) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 976 | 2.8 | 0.57× (0.57–0.58) | 2.99× (2.95–3.04) | 5.95× (5.85–6.05) | 3.57× (3.52–3.62) | 6.05× (5.96–6.16) | 2.96× (2.91–3.00) | 2.64× (2.60–2.68) | 1.00× (0.98–1.02) | 1.50× (1.47–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 978 | 2.8 | 0.58× (0.57–0.59) | 3.00× (2.93–3.06) | 5.98× (5.83–6.13) | 3.54× (3.47–3.62) | 6.09× (5.93–6.26) | 2.95× (2.89–3.01) | 2.63× (2.58–2.69) | 0.99× (0.97–1.02) | 1.49× (1.46–1.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.035 s | 558 | 4.6 | 2.79× (2.69–2.89) | 2.90× (2.87–2.93) | 3.72× (3.69–3.76) | 3.64× (3.61–3.67) | 3.96× (3.92–3.99) | 2.45× (2.42–2.47) | 2.03× (2.01–2.04) | 1.93× (1.92–1.95) | 2.89× (2.86–2.92) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 834 | 5.8 | 2.14× (1.75–2.60) | 2.54× (2.06–3.13) | 3.60× (3.11–4.16) | 2.89× (2.47–3.36) | 4.11× (3.50–4.82) | 3.31× (2.83–3.87) | 2.74× (2.34–3.21) | 2.62× (2.24–3.06) | 3.91× (3.34–4.57) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 757 | 11.0 | 1.82× (1.76–1.88) | 1.94× (1.80–2.08) | 2.92× (2.76–3.08) | 2.22× (2.17–2.28) | 3.77× (3.64–3.90) | 3.34× (3.23–3.44) | 2.76× (2.68–2.85) | 2.63× (2.55–2.72) | 3.94× (3.82–4.06) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.130 s | 62 | 40.3 | 1.37× (1.36–1.38) | 2.74× (2.72–2.76) | 2.46× (2.40–2.51) | 2.48× (2.47–2.49) | 2.88× (2.85–2.91) | 3.17× (3.13–3.21) | 3.15× (3.14–3.16) | 1.23× (1.22–1.24) | 1.41× (1.39–1.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.082 s | 98 | 45.8 | 1.24× (1.16–1.32) | 2.42× (2.28–2.58) | 2.30× (2.16–2.45) | 2.96× (2.75–3.19) | 2.81× (2.64–2.99) | 4.90× (4.59–5.23) | 4.87× (4.57–5.18) | 1.90× (1.79–2.03) | 2.18× (2.05–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.069 s | 116 | 62.7 | 1.32× (1.28–1.36) | 2.13× (2.09–2.16) | 2.05× (1.98–2.12) | 2.56× (2.50–2.63) | 2.82× (2.78–2.87) | 6.00× (5.92–6.08) | 5.96× (5.89–6.03) | 2.33× (2.31–2.36) | 2.67× (2.63–2.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 s | 66 | 37.8 | 1.47× (1.45–1.49) | 3.01× (2.97–3.04) | 2.51× (2.46–2.56) | 2.60× (2.58–2.63) | 3.03× (2.98–3.08) | 3.60× (3.56–3.64) | 3.82× (3.77–3.87) | 1.40× (1.38–1.43) | 1.70× (1.67–1.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.086 s | 111 | 42.3 | 1.40× (1.37–1.44) | 2.80× (2.73–2.87) | 2.51× (2.46–2.57) | 3.52× (3.40–3.65) | 3.12× (3.08–3.16) | 6.08× (6.01–6.16) | 6.46× (6.37–6.55) | 2.37× (2.34–2.41) | 2.87× (2.82–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 123 | 60.8 | 1.40× (1.36–1.43) | 2.16× (2.11–2.21) | 2.14× (2.10–2.18) | 2.91× (2.88–2.95) | 2.73× (2.68–2.77) | 6.72× (6.63–6.80) | 7.13× (7.05–7.21) | 2.62× (2.58–2.66) | 3.17× (3.13–3.21) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 741 | 3.8 | 0.67× (0.66–0.67) | 3.03× (3.01–3.06) | 4.45× (4.41–4.48) | 3.14× (3.12–3.16) | 6.77× (6.69–6.85) | 2.54× (2.52–2.56) | 2.33× (2.31–2.35) | 0.98× (0.97–0.99) | 1.53× (1.52–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 699 | 4.1 | 0.56× (0.55–0.57) | 2.88× (2.83–2.93) | 4.93× (4.86–5.01) | 2.99× (2.95–3.03) | 6.44× (6.32–6.55) | 2.40× (2.36–2.44) | 2.20× (2.17–2.23) | 0.93× (0.91–0.95) | 1.44× (1.42–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 690 | 4.0 | 0.56× (0.56–0.57) | 2.86× (2.83–2.89) | 4.95× (4.84–5.05) | 2.97× (2.93–3.00) | 6.35× (6.28–6.42) | 2.38× (2.36–2.40) | 2.18× (2.16–2.21) | 0.92× (0.91–0.93) | 1.43× (1.42–1.45) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.090 s | 138 | 18.1 | 1.47× (1.46–1.49) | 3.08× (3.04–3.12) | 3.34× (3.31–3.38) | 3.34× (3.30–3.38) | 3.52× (3.44–3.59) | 3.94× (3.90–3.98) | 3.58× (3.52–3.64) | 1.13× (1.12–1.15) | 1.38× (1.36–1.41) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.061 s | 203 | 21.9 | 1.21× (1.15–1.27) | 2.59× (2.42–2.77) | 2.90× (2.74–3.07) | 3.27× (3.15–3.39) | 3.18× (3.06–3.30) | 5.72× (5.52–5.94) | 5.19× (4.99–5.41) | 1.65× (1.58–1.71) | 2.01× (1.92–2.10) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.049 s | 253 | 31.5 | 1.27× (1.25–1.30) | 2.01× (1.93–2.09) | 2.57× (2.51–2.63) | 2.94× (2.82–3.06) | 3.09× (3.01–3.16) | 7.16× (7.00–7.33) | 6.50× (6.35–6.65) | 2.06× (2.01–2.11) | 2.52× (2.44–2.59) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.50× (2.49–2.51) | 5.61× (5.58–5.64) | 5.79× (5.76–5.82) | 6.29× (6.27–6.31) | 5.63× (5.58–5.68) | 6.16× (6.12–6.21) | 5.67× (5.62–5.72) | 1.83× (1.82–1.84) | 2.14× (2.13–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.38× (2.37–2.40) | 5.31× (5.27–5.35) | 5.65× (5.56–5.73) | 6.25× (6.11–6.38) | 5.26× (5.22–5.29) | 5.67× (5.63–5.72) | 5.22× (5.18–5.26) | 1.69× (1.68–1.70) | 1.97× (1.96–1.99) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.30× (2.27–2.33) | 3.83× (3.78–3.88) | 4.12× (4.06–4.19) | 4.35× (4.26–4.44) | 3.79× (3.72–3.87) | 3.43× (3.37–3.49) | 3.16× (3.11–3.21) | 1.02× (1.00–1.04) | 1.19× (1.17–1.22) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.26× (3.22–3.30) | 4.09× (4.00–4.18) | 3.32× (3.28–3.36) | 5.61× (5.54–5.68) | 2.78× (2.75–2.81) | 2.49× (2.47–2.52) | 1.00× (0.98–1.02) | 1.07× (1.04–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.07× (3.03–3.11) | 5.35× (5.28–5.42) | 3.31× (3.26–3.35) | 5.52× (5.42–5.61) | 2.73× (2.69–2.77) | 2.45× (2.41–2.49) | 0.98× (0.96–1.00) | 1.05× (1.03–1.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.62× (0.61–0.63) | 3.43× (3.34–3.53) | 5.44× (5.34–5.55) | 3.35× (3.28–3.41) | 5.63× (5.49–5.78) | 2.78× (2.72–2.83) | 2.49× (2.44–2.54) | 1.00× (0.97–1.02) | 1.07× (1.03–1.10) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.70× (2.60–2.79) | 2.68× (2.65–2.70) | 3.36× (3.33–3.39) | 3.50× (3.46–3.53) | 3.80× (3.77–3.84) | 2.37× (2.35–2.39) | 1.97× (1.95–1.99) | 1.89× (1.87–1.90) | 2.60× (2.57–2.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.16× (2.11–2.20) | 2.40× (2.33–2.46) | 2.90× (2.85–2.95) | 2.97× (2.91–3.03) | 3.17× (3.11–3.24) | 1.89× (1.86–1.93) | 1.57× (1.55–1.60) | 1.51× (1.48–1.53) | 2.08× (2.03–2.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.98× (1.94–2.03) | 1.68× (1.65–1.72) | 2.15× (2.11–2.19) | 2.02× (1.97–2.06) | 2.03× (1.99–2.07) | 1.00× (0.98–1.02) | 0.83× (0.82–0.85) | 0.80× (0.78–0.81) | 1.10× (1.08–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.66× (2.65–2.67) | 2.40× (2.36–2.44) | 2.46× (2.45–2.47) | 2.85× (2.82–2.89) | 3.14× (3.10–3.18) | 3.12× (3.11–3.13) | 1.23× (1.22–1.24) | 1.38× (1.38–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.28× (1.26–1.30) | 2.45× (2.41–2.50) | 2.17× (2.14–2.20) | 2.89× (2.85–2.93) | 2.55× (2.52–2.59) | 2.76× (2.70–2.82) | 2.74× (2.70–2.78) | 1.08× (1.06–1.10) | 1.22× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.38–1.55) | 2.31× (2.24–2.37) | 1.87× (1.81–1.92) | 2.56× (2.49–2.62) | 2.44× (2.39–2.49) | 2.04× (1.97–2.11) | 2.03× (1.98–2.08) | 0.80× (0.78–0.82) | 0.90× (0.88–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.45–1.48) | 2.95× (2.91–2.99) | 2.47× (2.43–2.51) | 2.58× (2.56–2.61) | 3.00× (2.95–3.05) | 3.56× (3.52–3.60) | 3.78× (3.73–3.83) | 1.40× (1.37–1.42) | 1.65× (1.63–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.37–1.40) | 2.75× (2.72–2.79) | 2.30× (2.29–2.31) | 3.20× (3.18–3.23) | 2.73× (2.71–2.75) | 3.17× (3.15–3.20) | 3.37× (3.34–3.40) | 1.25× (1.23–1.26) | 1.47× (1.46–1.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.62× (1.58–1.66) | 2.51× (2.46–2.56) | 1.92× (1.88–1.96) | 2.78× (2.72–2.84) | 2.50× (2.44–2.56) | 2.22× (2.18–2.26) | 2.35× (2.31–2.40) | 0.87× (0.85–0.89) | 1.03× (1.01–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.71) | 2.72× (2.69–2.74) | 3.56× (3.53–3.59) | 2.85× (2.84–2.87) | 6.01× (5.93–6.10) | 2.34× (2.33–2.35) | 2.15× (2.14–2.17) | 0.98× (0.97–0.99) | 0.99× (0.96–1.03) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.62–0.63) | 2.51× (2.48–2.54) | 4.25× (4.19–4.30) | 2.75× (2.73–2.78) | 5.66× (5.58–5.75) | 2.18× (2.15–2.20) | 2.00× (1.98–2.03) | 0.92× (0.90–0.93) | 0.92× (0.89–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.61–0.62) | 2.76× (2.73–2.80) | 4.35× (4.26–4.44) | 2.82× (2.78–2.85) | 5.74× (5.66–5.82) | 2.22× (2.19–2.25) | 2.05× (2.02–2.07) | 0.93× (0.92–0.95) | 0.94× (0.90–0.98) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.46× (1.45–1.48) | 2.98× (2.95–3.02) | 3.23× (3.19–3.27) | 3.29× (3.25–3.32) | 3.46× (3.39–3.54) | 3.88× (3.84–3.92) | 3.52× (3.46–3.58) | 1.13× (1.12–1.14) | 1.35× (1.33–1.36) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.31× (1.27–1.36) | 2.57× (2.50–2.63) | 2.82× (2.73–2.93) | 3.11× (3.03–3.18) | 2.92× (2.85–3.00) | 3.17× (3.09–3.26) | 2.88× (2.80–2.96) | 0.93× (0.90–0.95) | 1.10× (1.07–1.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.47× (1.41–1.54) | 2.23× (2.16–2.31) | 2.43× (2.37–2.50) | 2.62× (2.56–2.69) | 2.54× (2.49–2.60) | 2.23× (2.18–2.27) | 2.02× (1.98–2.07) | 0.65× (0.64–0.66) | 0.77× (0.76–0.79) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.34× (0.33–0.36) | 1.07× (1.07–1.07) | 0.86× (0.85–0.86) | 12.08× (12.07–12.09) | 0.29× (0.28–0.29) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.39) | 0.56× (0.51–0.62) | 0.80× (0.79–0.81) | 0.66× (0.65–0.68) | 6.37× (6.32–6.42) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.39) | 0.67× (0.61–0.74) | 0.70× (0.69–0.70) | 0.61× (0.60–0.62) | 4.08× (4.06–4.10) | 0.10× (0.09–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.30× (4.29–4.31) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.83–0.84) | 4.40× (4.40–4.41) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.41–4.41) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.03–8.04) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.65–0.68) | 0.71× (0.65–0.77) | 1.62× (1.59–1.64) | 0.80× (0.79–0.81) | 5.84× (5.75–5.94) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.67× (0.63–0.72) | 1.16× (1.13–1.19) | 0.70× (0.68–0.73) | 3.39× (3.30–3.50) | 0.27× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.19) | 0.74× (0.74–0.74) | 5.23× (5.21–5.24) | 0.33× (0.33–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.49–0.53) | 0.40× (0.36–0.44) | 0.93× (0.90–0.97) | 0.74× (0.71–0.78) | 3.23× (3.11–3.35) | 0.20× (0.19–0.21) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.47) | 0.88× (0.87–0.90) | 0.82× (0.79–0.86) | 2.57× (2.52–2.63) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.22–5.24) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.34× (0.32–0.36) | 0.85× (0.83–0.88) | 0.71× (0.68–0.74) | 2.93× (2.85–3.01) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.50× (0.47–0.53) | 0.80× (0.79–0.81) | 0.82× (0.78–0.86) | 2.32× (2.29–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.21) | 1.08× (1.08–1.09) | 0.77× (0.77–0.78) | 5.40× (5.39–5.41) | 0.35× (0.35–0.36) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.61–1.63) | 0.77× (0.77–0.77) | 5.35× (5.33–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.77× (0.77–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.41) | 1.47× (1.47–1.48) | 0.69× (0.69–0.70) | 6.64× (6.62–6.66) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 0.65× (0.65–0.66) | 0.33× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 38 MB | 0.44× (0.43–0.45) | 0.41× (0.39–0.44) | 1.05× (1.03–1.07) | 0.67× (0.65–0.69) | 3.27× (3.20–3.33) | 0.20× (0.19–0.20) | 0.21× (0.21–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.16) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.52× (0.48–0.56) | 0.95× (0.92–0.97) | 0.75× (0.71–0.79) | 2.41× (2.35–2.46) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 343 | 2.92 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 624 | 3.17 | 1.09× (1.08–1.09) | 89% | 1.03× (1.02–1.04) | 1.03× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 717 | 5.26 | 1.79× (1.77–1.82) | 52% | 1.22× (1.21–1.24) | 1.65× (1.63–1.67) |

**Best:** mibz2 at 4 threads (717 MB/s); lbzip2 at 4 threads (194 MB/s); pbz2 at 4 threads (296 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.70, mibz2 4t 1.08, lbzip2 1t 0.09, lbzip2 2t 0.33, lbzip2 4t 0.54, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 0.90 | 0.02 | 0.08 | 0.292 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.85 | 12.5% |
| MTF + RLE2 | 1.20 | 17.7% |
| T-vector build | 1.14 | 16.8% |
| walk (the one each build uses) | 3.16 | 46.7% |
| of which: segment assembly | 0.11 | 1.7% |
| RLE1 | 0.22 | 3.2% |
| CRC | 0.19 | 2.9% |
| **total** | 6.76 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.489 | 1.300 | (in its total) | 2.66× (2.65–2.67) |  |
| walk | 1.223 | 2.631 | (in its total) | 2.15× (2.13–2.16) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.057 | (in its total) | 1.30× (1.28–1.32) |  |
| inverse BWT, total | 1.756 | 3.988 | 3.220 | 2.27× (2.26–2.28) | 1.83× (1.82–1.84) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h28m ± 10m15s |
| 4 threads | 42m40s ± 6m00s |
| 4 threads, the sample's overlap gap as a fixed cost | 32m18s ± 4m34s |
| the scanner alone (its serial part) | 15m43s ± 26.3s |
| the workers alone, over 4 | 32m18s ± 4m34s |

At 4 threads the workers bound it. Per byte: the scanner 9.1 ns a byte of bz2, the workers 4.24 ns a byte of output at 4 threads (CPU, summed over them), 2.40 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
