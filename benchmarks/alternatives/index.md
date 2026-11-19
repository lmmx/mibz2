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

The latest record: [`bd08619`](<https://github.com/lmmx/mibz2/commit/bd0861939dbae52f6d171fc7a8068779602edfd7>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44571663113>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.511 s | 410 | 7.2 | 3.03× (2.99–3.07) | 6.86× (6.80–6.91) | 7.10× (7.02–7.18) | 7.69× (7.59–7.80) | 6.91× (6.84–6.99) | 7.54× (7.47–7.61) | 6.95× (6.86–7.04) | 2.22× (2.19–2.25) | 2.59× (2.56–2.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.291 s | 721 | 7.8 | 2.95× (2.83–3.07) | 6.39× (6.16–6.63) | 7.30× (7.03–7.58) | 7.02× (6.77–7.27) | 6.39× (6.14–6.66) | 12.97× (12.48–13.49) | 11.96× (11.52–12.42) | 3.81× (3.67–3.96) | 4.45× (4.26–4.65) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.238 s | 879 | 12.2 | 3.11× (2.98–3.24) | 4.64× (4.54–4.74) | 5.79× (5.65–5.94) | 5.41× (5.11–5.73) | 5.43× (5.22–5.65) | 16.07× (15.90–16.25) | 14.82× (14.60–15.04) | 4.73× (4.65–4.80) | 5.51× (5.43–5.60) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 978 | 3.3 | 0.60× (0.59–0.60) | 3.68× (3.52–3.86) | 4.50× (4.39–4.61) | 3.57× (3.52–3.61) | 6.20× (6.12–6.29) | 2.99× (2.92–3.06) | 2.66× (2.62–2.69) | 1.02× (1.00–1.04) | 1.50× (1.48–1.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 937 | 3.5 | 0.56× (0.55–0.57) | 2.90× (2.83–2.97) | 5.95× (5.78–6.12) | 3.46× (3.38–3.54) | 5.94× (5.82–6.06) | 2.89× (2.81–2.96) | 2.57× (2.50–2.63) | 0.99× (0.95–1.02) | 1.45× (1.41–1.49) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 s | 920 | 3.5 | 0.56× (0.55–0.57) | 2.90× (2.83–2.98) | 5.88× (5.74–6.02) | 3.44× (3.35–3.52) | 5.91× (5.80–6.03) | 2.88× (2.78–2.98) | 2.56× (2.49–2.62) | 0.98× (0.95–1.02) | 1.44× (1.41–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 574 | 5.4 | 2.97× (2.87–3.07) | 2.99× (2.93–3.04) | 3.83× (3.76–3.90) | 3.80× (3.74–3.86) | 4.11× (4.05–4.16) | 2.54× (2.49–2.59) | 2.09× (2.06–2.12) | 2.01× (1.98–2.04) | 3.00× (2.94–3.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 875 | 6.8 | 2.08× (1.87–2.32) | 2.78× (2.33–3.32) | 3.85× (3.41–4.34) | 3.18× (2.80–3.61) | 4.43× (3.94–4.97) | 3.58× (3.17–4.04) | 2.95× (2.62–3.31) | 2.83× (2.51–3.19) | 4.23× (3.74–4.79) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 761 | 13.3 | 1.88× (1.78–1.99) | 1.88× (1.75–2.02) | 2.95× (2.79–3.12) | 2.13× (2.06–2.21) | 3.85× (3.72–4.00) | 3.36× (3.24–3.48) | 2.77× (2.68–2.86) | 2.66× (2.57–2.75) | 3.98× (3.83–4.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.135 s | 59 | 50.2 | 1.33× (1.32–1.34) | 2.64× (2.60–2.67) | 2.41× (2.37–2.45) | 2.45× (2.42–2.48) | 2.82× (2.78–2.86) | 3.09× (3.06–3.12) | 3.09× (3.04–3.13) | 1.20× (1.19–1.21) | 1.39× (1.36–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 s | 96 | 56.4 | 1.25× (1.17–1.33) | 2.46× (2.30–2.64) | 2.36× (2.19–2.54) | 2.94× (2.75–3.13) | 2.83× (2.66–3.00) | 4.92× (4.64–5.23) | 4.91× (4.61–5.23) | 1.92× (1.80–2.03) | 2.21× (2.08–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 113 | 77.0 | 1.31× (1.24–1.39) | 2.10× (2.01–2.19) | 2.02× (1.95–2.09) | 2.52× (2.43–2.61) | 2.77× (2.70–2.84) | 5.87× (5.73–6.02) | 5.86× (5.72–6.00) | 2.29× (2.21–2.36) | 2.64× (2.56–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.151 s | 64 | 46.5 | 1.45× (1.43–1.47) | 3.00× (2.97–3.03) | 2.58× (2.47–2.70) | 2.61× (2.57–2.64) | 3.09× (3.04–3.14) | 3.56× (3.52–3.60) | 3.80× (3.75–3.84) | 1.39× (1.36–1.41) | 1.67× (1.65–1.70) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.091 s | 105 | 52.4 | 1.37× (1.27–1.47) | 2.73× (2.64–2.81) | 2.40× (2.31–2.49) | 3.36× (3.26–3.46) | 3.01× (2.90–3.12) | 5.84× (5.66–6.01) | 6.22× (6.02–6.43) | 2.27× (2.19–2.36) | 2.74× (2.64–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.079 s | 121 | 73.5 | 1.40× (1.35–1.44) | 2.25× (2.15–2.36) | 2.14× (2.05–2.24) | 2.90× (2.80–3.00) | 2.72× (2.64–2.80) | 6.69× (6.54–6.85) | 7.14× (6.94–7.34) | 2.60× (2.55–2.66) | 3.14× (3.09–3.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 735 | 4.6 | 0.66× (0.65–0.67) | 3.35× (3.08–3.64) | 4.43× (4.30–4.57) | 3.10× (3.06–3.15) | 6.81× (6.71–6.92) | 2.50× (2.47–2.54) | 2.30× (2.27–2.33) | 0.98× (0.96–1.00) | 1.51× (1.49–1.53) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 702 | 4.9 | 0.57× (0.56–0.57) | 2.89× (2.84–2.93) | 5.01× (4.84–5.19) | 3.00× (2.95–3.06) | 6.51× (6.40–6.62) | 2.41× (2.37–2.44) | 2.21× (2.17–2.25) | 0.94× (0.92–0.95) | 1.45× (1.43–1.47) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 683 | 4.9 | 0.57× (0.55–0.59) | 2.82× (2.76–2.88) | 4.94× (4.73–5.16) | 2.93× (2.88–2.98) | 6.31× (6.20–6.43) | 2.34× (2.29–2.39) | 2.15× (2.10–2.20) | 0.91× (0.90–0.93) | 1.41× (1.38–1.44) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.071 s | 183 | 16.4 | 1.97× (1.94–1.99) | 4.13× (4.08–4.18) | 4.50× (4.45–4.55) | 4.50× (4.44–4.56) | 4.69× (4.63–4.74) | 5.23× (5.16–5.30) | 4.74× (4.67–4.80) | 1.52× (1.48–1.56) | 1.85× (1.83–1.88) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.049 s | 269 | 20.4 | 1.68× (1.59–1.77) | 3.35× (3.30–3.39) | 3.97× (3.78–4.17) | 4.14× (4.01–4.26) | 4.24× (4.10–4.38) | 7.44× (7.18–7.70) | 6.74× (6.48–7.00) | 2.16× (2.06–2.27) | 2.63× (2.54–2.73) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.045 s | 291 | 31.5 | 1.57× (1.50–1.64) | 2.51× (2.36–2.67) | 3.33× (3.11–3.56) | 3.25× (3.14–3.36) | 3.77× (3.65–3.90) | 8.32× (8.09–8.55) | 7.53× (7.36–7.71) | 2.42× (2.34–2.50) | 2.94× (2.87–3.02) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.02× (2.98–3.06) | 6.83× (6.77–6.88) | 7.06× (6.99–7.14) | 7.67× (7.56–7.77) | 6.89× (6.82–6.96) | 7.51× (7.44–7.58) | 6.93× (6.84–7.01) | 2.21× (2.18–2.24) | 2.57× (2.54–2.60) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.89× (2.85–2.93) | 6.43× (6.38–6.48) | 6.79× (6.73–6.84) | 7.38× (7.32–7.44) | 6.40× (6.31–6.49) | 6.89× (6.84–6.93) | 6.35× (6.29–6.41) | 2.03× (2.01–2.05) | 2.36× (2.33–2.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.95× (2.86–3.03) | 4.96× (4.83–5.09) | 5.39× (5.22–5.56) | 5.68× (5.49–5.88) | 5.13× (4.89–5.37) | 4.42× (4.35–4.49) | 4.08× (4.00–4.15) | 1.30× (1.28–1.32) | 1.51× (1.49–1.53) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.63–0.64) | 3.28× (3.24–3.32) | 4.07× (4.04–4.11) | 3.34× (3.30–3.38) | 5.74× (5.66–5.83) | 2.81× (2.74–2.89) | 2.51× (2.48–2.54) | 1.02× (1.01–1.04) | 1.07× (1.04–1.10) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 3.00× (2.93–3.07) | 5.25× (5.14–5.37) | 3.21× (3.14–3.28) | 5.42× (5.33–5.52) | 2.67× (2.61–2.74) | 2.39× (2.33–2.45) | 0.97× (0.94–1.01) | 1.02× (0.99–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.32× (3.24–3.41) | 5.33× (5.21–5.44) | 3.25× (3.17–3.33) | 5.47× (5.37–5.58) | 2.71× (2.62–2.81) | 2.42× (2.36–2.48) | 0.98× (0.95–1.02) | 1.03× (1.00–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.86× (2.77–2.95) | 2.76× (2.71–2.82) | 3.48× (3.44–3.53) | 3.65× (3.59–3.71) | 3.94× (3.89–4.00) | 2.46× (2.41–2.50) | 2.03× (2.00–2.07) | 1.96× (1.93–1.98) | 2.72× (2.65–2.79) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.17× (2.09–2.25) | 2.42× (2.31–2.53) | 2.97× (2.89–3.07) | 3.02× (2.90–3.14) | 3.21× (3.08–3.35) | 1.92× (1.83–2.02) | 1.59× (1.52–1.66) | 1.53× (1.46–1.60) | 2.13× (2.02–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.91× (1.83–2.00) | 1.67× (1.61–1.74) | 2.13× (2.06–2.20) | 2.00× (1.93–2.07) | 2.04× (1.97–2.10) | 1.00× (0.96–1.03) | 0.83× (0.80–0.85) | 0.80× (0.77–0.82) | 1.11× (1.07–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.34) | 2.58× (2.56–2.61) | 2.35× (2.32–2.39) | 2.42× (2.40–2.45) | 2.80× (2.76–2.84) | 3.06× (3.03–3.09) | 3.05× (3.01–3.10) | 1.20× (1.19–1.21) | 1.35× (1.33–1.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.28) | 2.45× (2.37–2.52) | 2.19× (2.12–2.26) | 2.85× (2.81–2.88) | 2.52× (2.48–2.57) | 2.72× (2.69–2.75) | 2.72× (2.67–2.76) | 1.07× (1.06–1.08) | 1.20× (1.18–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.39–1.47) | 2.23× (2.16–2.31) | 1.84× (1.79–1.89) | 2.50× (2.45–2.56) | 2.40× (2.35–2.44) | 1.99× (1.96–2.03) | 1.99× (1.96–2.02) | 0.78× (0.77–0.80) | 0.88× (0.86–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.44× (1.43–1.46) | 2.94× (2.91–2.98) | 2.52× (2.41–2.63) | 2.59× (2.55–2.62) | 3.06× (3.01–3.11) | 3.52× (3.49–3.56) | 3.76× (3.71–3.80) | 1.38× (1.36–1.40) | 1.63× (1.61–1.65) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.33× (1.29–1.36) | 2.66× (2.59–2.75) | 2.22× (2.16–2.28) | 3.11× (3.04–3.18) | 2.67× (2.58–2.77) | 3.07× (2.98–3.16) | 3.27× (3.17–3.38) | 1.20× (1.16–1.25) | 1.42× (1.37–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.52–1.63) | 2.45× (2.37–2.54) | 1.89× (1.83–1.96) | 2.72× (2.63–2.80) | 2.47× (2.40–2.55) | 2.19× (2.14–2.24) | 2.33× (2.27–2.40) | 0.86× (0.83–0.88) | 1.01× (0.98–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.76× (2.67–2.84) | 3.60× (3.52–3.69) | 2.83× (2.80–2.87) | 6.07× (5.99–6.15) | 2.31× (2.29–2.34) | 2.13× (2.11–2.15) | 0.98× (0.97–1.00) | 1.00× (0.96–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.52× (2.48–2.57) | 4.29× (4.22–4.37) | 2.77× (2.72–2.82) | 5.74× (5.64–5.84) | 2.19× (2.15–2.23) | 2.02× (1.98–2.06) | 0.93× (0.91–0.95) | 0.95× (0.91–0.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.60–0.64) | 2.71× (2.67–2.76) | 4.26× (4.17–4.35) | 2.77× (2.73–2.81) | 5.70× (5.61–5.79) | 2.18× (2.14–2.22) | 2.01× (1.97–2.05) | 0.93× (0.91–0.94) | 0.94× (0.91–0.97) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.94× (1.92–1.96) | 3.95× (3.91–3.99) | 4.35× (4.29–4.40) | 4.40× (4.34–4.46) | 4.59× (4.54–4.63) | 5.11× (5.05–5.18) | 4.63× (4.57–4.69) | 1.51× (1.47–1.54) | 1.79× (1.77–1.81) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.70× (1.66–1.74) | 3.34× (3.28–3.40) | 3.65× (3.59–3.72) | 4.05× (4.00–4.10) | 3.78× (3.71–3.84) | 4.10× (4.05–4.15) | 3.71× (3.67–3.75) | 1.21× (1.18–1.24) | 1.43× (1.41–1.45) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.67× (1.60–1.74) | 2.67× (2.60–2.75) | 2.94× (2.81–3.08) | 3.20× (3.13–3.28) | 3.02× (2.93–3.11) | 2.66× (2.62–2.70) | 2.41× (2.37–2.45) | 0.78× (0.76–0.81) | 0.93× (0.91–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.30× (0.29–0.31) | 0.98× (0.97–0.98) | 0.78× (0.78–0.78) | 11.01× (10.97–11.05) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 57 MB | 0.33× (0.32–0.34) | 0.50× (0.46–0.56) | 0.69× (0.67–0.71) | 0.57× (0.56–0.59) | 5.45× (5.34–5.57) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) | 0.21× (0.20–0.21) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.33× (0.32–0.34) | 0.64× (0.57–0.72) | 0.59× (0.57–0.61) | 0.53× (0.49–0.56) | 3.43× (3.30–3.56) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.13) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.38× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.66× (0.65–0.67) | 8.04× (8.02–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.64–0.67) | 0.68× (0.61–0.76) | 1.57× (1.54–1.61) | 0.79× (0.78–0.81) | 5.71× (5.59–5.83) | 0.46× (0.45–0.47) | 0.48× (0.47–0.50) | 0.52× (0.51–0.54) | 0.26× (0.26–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.66× (0.64–0.67) | 0.71× (0.65–0.77) | 1.17× (1.15–1.20) | 0.72× (0.71–0.73) | 3.45× (3.39–3.50) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.73) | 5.17× (5.16–5.19) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.52× (0.51–0.54) | 0.39× (0.36–0.42) | 0.93× (0.90–0.95) | 0.75× (0.73–0.76) | 3.19× (3.11–3.26) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.67) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.82× (0.79–0.84) | 2.51× (2.45–2.57) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.18× (1.17–1.18) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.33–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.38× (0.35–0.41) | 0.84× (0.81–0.87) | 0.70× (0.67–0.73) | 2.89× (2.80–2.98) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.49× (0.46–0.52) | 0.80× (0.79–0.81) | 0.78× (0.76–0.81) | 2.31× (2.28–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.38× (5.37–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.77–0.77) | 5.29× (5.27–5.30) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.31–5.33) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.43× (1.42–1.43) | 0.67× (0.66–0.67) | 6.45× (6.43–6.48) | 0.39× (0.39–0.40) | 0.42× (0.41–0.43) | 0.63× (0.63–0.63) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 38 MB | 0.45× (0.44–0.45) | 0.43× (0.41–0.46) | 1.06× (1.04–1.08) | 0.69× (0.67–0.71) | 3.29× (3.25–3.34) | 0.20× (0.20–0.20) | 0.21× (0.21–0.22) | 0.32× (0.32–0.32) | 0.16× (0.16–0.16) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 55 MB | 0.50× (0.48–0.52) | 0.51× (0.47–0.56) | 0.92× (0.89–0.94) | 0.76× (0.74–0.78) | 2.34× (2.28–2.40) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 410 | 2.45 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 721 | 2.66 | 1.09× (1.08–1.10) | 86% | 1.03× (1.02–1.04) | 1.04× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 879 | 4.18 | 1.70× (1.67–1.73) | 53% | 1.23× (1.21–1.26) | 1.66× (1.62–1.69) |

**Best:** mibz2 at 4 threads (879 MB/s); lbzip2 at 4 threads (189 MB/s); pbz2 at 4 threads (285 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.18, lbzip2 1t 0.09, lbzip2 2t 0.39, lbzip2 4t 0.58, pbz2 1t 0.00, pbz2 2t 0.29, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 11%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.67 | 0.02 | 0.09 | 0.238 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2922 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.01 | 12.1% |
| MTF + RLE2 | 1.43 | 17.0% |
| T-vector build | 1.38 | 16.4% |
| walk (the one each build uses) | 4.07 | 48.5% |
| of which: segment assembly | 0.16 | 1.9% |
| RLE1 | 0.26 | 3.0% |
| CRC | 0.23 | 2.8% |
| **total** | 8.40 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.492 | 1.307 | (in its total) | 2.66× (2.65–2.68) |  |
| walk | 1.286 | 2.674 | (in its total) | 2.08× (2.06–2.10) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.049 | 0.059 | (in its total) | 1.20× (1.17–1.23) |  |
| inverse BWT, total | 1.827 | 4.040 | 3.288 | 2.21× (2.19–2.23) | 1.80× (1.79–1.81) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h14m ± 8m32s |
| 4 threads | 34m53s ± 4m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m16s ± 3m25s |
| the scanner alone (its serial part) | 13m41s ± 32.2s |
| the workers alone, over 4 | 24m16s ± 3m25s |

At 4 threads the workers bound it. Per byte: the scanner 7.9 ns a byte of bz2, the workers 3.18 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2922.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
