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

The latest record: [`88768ac`](<https://github.com/lmmx/mibz2/commit/88768ac15924b08ceb039169690c942542791f7e>) (2026-11-10).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42787233888>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.6.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.629 s | 333 | 8.3 | 2.45× (2.43–2.48) | 5.67× (5.60–5.74) | 5.82× (5.79–5.85) | 6.27× (6.23–6.32) | 5.65× (5.61–5.68) | 6.14× (6.11–6.18) | 5.67× (5.64–5.71) | 1.82× (1.81–1.83) | 2.08× (2.06–2.11) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.368 s | 570 | 9.0 | 2.44× (2.29–2.61) | 5.15× (4.89–5.43) | 5.84× (5.59–6.11) | 5.75× (5.50–6.01) | 5.17× (4.98–5.37) | 10.50× (10.09–10.93) | 9.70× (9.32–10.10) | 3.11× (2.98–3.25) | 3.56× (3.43–3.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.297 s | 706 | 14.8 | 2.46× (2.42–2.50) | 3.77× (3.72–3.81) | 4.75× (4.65–4.85) | 4.38× (4.08–4.70) | 4.31× (4.24–4.37) | 12.96× (12.75–13.17) | 11.97× (11.79–12.16) | 3.84× (3.77–3.90) | 4.40× (4.34–4.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 971 | 3.1 | 0.59× (0.57–0.62) | 3.71× (3.44–4.01) | 4.59× (4.18–5.05) | 3.50× (3.36–3.65) | 6.07× (5.86–6.30) | 2.92× (2.81–3.04) | 2.61× (2.51–2.72) | 1.02× (0.97–1.06) | 1.48× (1.42–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 937 | 3.3 | 0.56× (0.55–0.57) | 2.89× (2.85–2.94) | 5.94× (5.77–6.11) | 3.47× (3.41–3.53) | 5.92× (5.84–6.01) | 2.86× (2.81–2.92) | 2.56× (2.51–2.61) | 1.00× (0.97–1.02) | 1.45× (1.42–1.47) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 941 | 3.2 | 0.56× (0.55–0.58) | 2.91× (2.86–2.97) | 5.95× (5.71–6.21) | 3.46× (3.39–3.53) | 5.95× (5.82–6.09) | 2.87× (2.82–2.92) | 2.57× (2.52–2.62) | 1.00× (0.98–1.02) | 1.45× (1.42–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 548 | 5.3 | 2.54× (2.46–2.62) | 2.85× (2.81–2.89) | 3.65× (3.60–3.71) | 3.59× (3.55–3.64) | 3.92× (3.86–3.99) | 2.41× (2.37–2.44) | 2.00× (1.97–2.03) | 1.92× (1.88–1.95) | 2.84× (2.79–2.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.023 s | 839 | 6.6 | 2.27× (2.10–2.46) | 2.82× (2.64–3.01) | 4.00× (3.87–4.13) | 3.23× (3.15–3.32) | 4.63× (4.54–4.73) | 3.69× (3.61–3.78) | 3.07× (3.01–3.14) | 2.94× (2.87–3.02) | 4.35× (4.25–4.45) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.027 s | 738 | 12.8 | 1.79× (1.72–1.87) | 1.95× (1.83–2.07) | 2.83× (2.66–3.00) | 2.07× (1.99–2.16) | 3.73× (3.53–3.93) | 3.23× (3.12–3.34) | 2.69× (2.60–2.79) | 2.58× (2.48–2.67) | 3.81× (3.67–3.95) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.131 s | 61 | 45.9 | 1.38× (1.37–1.39) | 2.74× (2.69–2.78) | 2.46× (2.42–2.50) | 2.50× (2.47–2.52) | 2.88× (2.86–2.90) | 3.17× (3.15–3.19) | 3.15× (3.12–3.18) | 1.24× (1.22–1.25) | 1.43× (1.40–1.46) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.081 s | 98 | 52.6 | 1.28× (1.21–1.35) | 2.48× (2.33–2.64) | 2.34× (2.21–2.48) | 2.97× (2.82–3.13) | 2.86× (2.70–3.02) | 4.94× (4.68–5.21) | 4.92× (4.67–5.18) | 1.93× (1.83–2.03) | 2.23× (2.12–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.071 s | 112 | 70.9 | 1.29× (1.25–1.32) | 2.11× (2.05–2.18) | 2.06× (1.99–2.14) | 2.58× (2.42–2.76) | 2.79× (2.72–2.87) | 5.90× (5.77–6.03) | 5.87× (5.74–6.01) | 2.30× (2.25–2.36) | 2.67× (2.58–2.75) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 s | 65 | 43.2 | 1.48× (1.46–1.49) | 3.06× (3.02–3.10) | 2.59× (2.55–2.63) | 2.65× (2.58–2.73) | 3.06× (3.03–3.09) | 3.61× (3.57–3.65) | 3.89× (3.85–3.93) | 1.43× (1.41–1.45) | 1.72× (1.69–1.74) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 s | 108 | 48.6 | 1.40× (1.35–1.46) | 2.83× (2.75–2.90) | 2.52× (2.48–2.56) | 3.45× (3.40–3.50) | 3.09× (3.06–3.12) | 6.00× (5.93–6.07) | 6.46× (6.40–6.53) | 2.38× (2.33–2.42) | 2.85× (2.80–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.080 s | 121 | 70.9 | 1.43× (1.40–1.45) | 2.21× (2.15–2.27) | 2.21× (2.14–2.27) | 2.95× (2.88–3.03) | 2.78× (2.71–2.85) | 6.74× (6.59–6.90) | 7.26× (7.11–7.42) | 2.67× (2.60–2.74) | 3.20× (3.12–3.28) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 719 | 4.4 | 0.66× (0.65–0.66) | 3.06× (2.90–3.23) | 4.34× (4.28–4.40) | 3.08× (3.04–3.12) | 6.73× (6.64–6.81) | 2.51× (2.47–2.54) | 2.29× (2.26–2.32) | 0.99× (0.96–1.01) | 1.50× (1.48–1.52) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 673 | 4.8 | 0.55× (0.54–0.56) | 2.79× (2.74–2.84) | 4.90× (4.75–5.05) | 2.92× (2.87–2.97) | 6.28× (6.16–6.39) | 2.34× (2.30–2.38) | 2.14× (2.10–2.18) | 0.92× (0.90–0.94) | 1.40× (1.37–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 676 | 4.6 | 0.56× (0.55–0.58) | 2.82× (2.77–2.86) | 4.86× (4.67–5.06) | 2.93× (2.89–2.98) | 6.30× (6.18–6.43) | 2.35× (2.31–2.40) | 2.15× (2.11–2.19) | 0.93× (0.90–0.95) | 1.41× (1.38–1.43) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 0.091 s | 136 | 20.7 | 1.47× (1.46–1.48) | 3.06× (3.01–3.11) | 3.35× (3.31–3.39) | 3.35× (3.31–3.39) | 3.56× (3.51–3.60) | 3.93× (3.88–3.98) | 3.55× (3.51–3.60) | 1.13× (1.12–1.15) | 1.43× (1.40–1.46) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 0.060 s | 207 | 25.0 | 1.21× (1.12–1.32) | 2.48× (2.24–2.74) | 2.96× (2.67–3.29) | 3.26× (2.96–3.59) | 3.19× (2.89–3.51) | 5.71× (5.19–6.29) | 5.17× (4.69–5.70) | 1.65× (1.49–1.82) | 2.08× (1.89–2.29) | yes |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 0.049 s | 252 | 35.4 | 1.32× (1.27–1.37) | 2.15× (2.08–2.23) | 2.67× (2.51–2.85) | 3.04× (2.95–3.14) | 3.24× (3.12–3.36) | 7.25× (7.08–7.41) | 6.55× (6.40–6.72) | 2.09× (2.03–2.15) | 2.64× (2.56–2.72) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.45× (2.42–2.47) | 5.65× (5.58–5.72) | 5.80× (5.77–5.82) | 6.25× (6.21–6.30) | 5.63× (5.60–5.67) | 6.12× (6.09–6.16) | 5.66× (5.62–5.69) | 1.81× (1.80–1.83) | 2.07× (2.05–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.36× (2.33–2.39) | 5.34× (5.28–5.40) | 5.61× (5.55–5.67) | 6.09× (6.04–6.14) | 5.24× (5.21–5.28) | 5.65× (5.60–5.70) | 5.22× (5.18–5.26) | 1.68× (1.66–1.69) | 1.91× (1.90–1.93) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.29× (2.25–2.33) | 3.90× (3.82–3.97) | 4.25× (4.18–4.33) | 4.42× (4.31–4.55) | 3.94× (3.86–4.03) | 3.44× (3.38–3.50) | 3.18× (3.13–3.23) | 1.02× (1.00–1.04) | 1.17× (1.15–1.18) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.60–0.65) | 3.22× (3.09–3.35) | 4.14× (3.85–4.46) | 3.28× (3.16–3.41) | 5.62× (5.44–5.82) | 2.75× (2.65–2.86) | 2.47× (2.38–2.56) | 1.01× (0.97–1.06) | 1.03× (0.98–1.09) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.62) | 2.99× (2.96–3.03) | 5.26× (5.19–5.32) | 3.22× (3.17–3.27) | 5.42× (5.35–5.48) | 2.66× (2.62–2.70) | 2.38× (2.35–2.42) | 0.98× (0.96–1.00) | 1.00× (0.97–1.03) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.62) | 3.33× (3.26–3.40) | 5.33× (5.22–5.44) | 3.28× (3.22–3.34) | 5.52× (5.40–5.65) | 2.71× (2.67–2.76) | 2.43× (2.39–2.48) | 1.00× (0.98–1.02) | 1.02× (0.98–1.06) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.47× (2.39–2.54) | 2.65× (2.62–2.68) | 3.34× (3.31–3.37) | 3.46× (3.43–3.50) | 3.78× (3.73–3.83) | 2.34× (2.31–2.36) | 1.96× (1.94–1.98) | 1.88× (1.85–1.91) | 2.58× (2.54–2.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.12× (2.07–2.17) | 2.39× (2.28–2.50) | 2.89× (2.82–2.97) | 2.94× (2.87–3.02) | 3.15× (3.09–3.22) | 1.86× (1.82–1.91) | 1.56× (1.53–1.60) | 1.50× (1.46–1.54) | 2.06× (2.00–2.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.94× (1.89–2.00) | 1.68× (1.62–1.74) | 2.11× (2.02–2.21) | 2.03× (1.96–2.10) | 1.99× (1.91–2.07) | 0.98× (0.94–1.01) | 0.82× (0.79–0.85) | 0.78× (0.76–0.81) | 1.08× (1.04–1.12) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.36–1.38) | 2.68× (2.64–2.73) | 2.41× (2.37–2.45) | 2.47× (2.45–2.50) | 2.85× (2.83–2.88) | 3.13× (3.11–3.15) | 3.12× (3.09–3.15) | 1.23× (1.22–1.25) | 1.39× (1.37–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.27–1.30) | 2.48× (2.46–2.50) | 2.18× (2.15–2.21) | 2.90× (2.86–2.93) | 2.58× (2.54–2.61) | 2.75× (2.72–2.78) | 2.74× (2.71–2.77) | 1.08× (1.07–1.10) | 1.22× (1.20–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.45–1.51) | 2.30× (2.23–2.37) | 1.89× (1.85–1.93) | 2.60× (2.52–2.68) | 2.47× (2.40–2.54) | 2.03× (2.00–2.06) | 2.02× (1.99–2.05) | 0.80× (0.78–0.81) | 0.90× (0.88–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.47× (1.46–1.49) | 2.99× (2.95–3.03) | 2.53× (2.50–2.57) | 2.63× (2.55–2.70) | 3.04× (3.00–3.07) | 3.58× (3.54–3.62) | 3.85× (3.81–3.89) | 1.42× (1.40–1.45) | 1.67× (1.65–1.69) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.41× (1.35–1.47) | 2.77× (2.74–2.81) | 2.32× (2.29–2.36) | 3.21× (3.17–3.25) | 2.74× (2.73–2.76) | 3.17× (3.16–3.19) | 3.41× (3.39–3.44) | 1.26× (1.24–1.29) | 1.48× (1.46–1.50) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.60× (1.58–1.62) | 2.51× (2.45–2.58) | 1.93× (1.89–1.96) | 2.81× (2.72–2.90) | 2.52× (2.45–2.60) | 2.20× (2.15–2.25) | 2.37× (2.32–2.42) | 0.88× (0.85–0.90) | 1.03× (1.00–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.70× (0.69–0.71) | 2.69× (2.65–2.73) | 3.52× (3.48–3.57) | 2.81× (2.77–2.85) | 5.99× (5.92–6.07) | 2.31× (2.28–2.34) | 2.12× (2.10–2.15) | 0.99× (0.97–1.01) | 0.99× (0.95–1.04) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.46× (2.43–2.50) | 4.21× (4.16–4.25) | 2.70× (2.66–2.75) | 5.55× (5.45–5.65) | 2.14× (2.10–2.17) | 1.96× (1.93–2.00) | 0.92× (0.90–0.93) | 0.92× (0.87–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.75× (2.70–2.79) | 4.28× (4.19–4.36) | 2.79× (2.75–2.84) | 5.71× (5.59–5.83) | 2.20× (2.15–2.24) | 2.02× (1.98–2.05) | 0.94× (0.92–0.96) | 0.94× (0.91–0.98) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 1.46× (1.45–1.47) | 2.98× (2.95–3.02) | 3.26× (3.24–3.29) | 3.30× (3.26–3.34) | 3.50× (3.46–3.55) | 3.87× (3.82–3.92) | 3.50× (3.45–3.55) | 1.13× (1.12–1.15) | 1.35× (1.33–1.37) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 1.34× (1.31–1.38) | 2.63× (2.59–2.66) | 2.89× (2.84–2.94) | 3.17× (3.13–3.21) | 2.99× (2.95–3.02) | 3.22× (3.20–3.24) | 2.92× (2.89–2.94) | 0.94× (0.93–0.96) | 1.13× (1.10–1.15) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 1.49× (1.43–1.56) | 2.33× (2.25–2.41) | 2.64× (2.50–2.79) | 2.73× (2.64–2.81) | 2.70× (2.62–2.78) | 2.26× (2.23–2.30) | 2.05× (2.02–2.08) | 0.66× (0.65–0.67) | 0.79× (0.77–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.33× (0.33–0.33) | 0.33× (0.31–0.35) | 1.07× (1.07–1.07) | 0.86× (0.86–0.86) | 12.09× (12.06–12.12) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 0.46× (0.46–0.47) | 0.23× (0.23–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.59× (0.52–0.66) | 0.80× (0.79–0.81) | 0.66× (0.66–0.67) | 6.35× (6.32–6.39) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.40) | 0.69× (0.60–0.80) | 0.70× (0.69–0.72) | 0.62× (0.60–0.65) | 4.10× (4.07–4.12) | 0.10× (0.10–0.10) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.29–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.27) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.78× (1.78–1.79) | 0.84× (0.84–0.84) | 4.41× (4.40–4.41) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.67× (0.66–0.67) | 8.04× (8.03–8.06) | 0.66× (0.65–0.66) | 0.68× (0.68–0.68) | 0.74× (0.73–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.77× (0.72–0.83) | 1.63× (1.59–1.67) | 0.80× (0.78–0.82) | 5.90× (5.78–6.02) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.67) | 0.72× (0.67–0.78) | 1.15× (1.09–1.21) | 0.70× (0.66–0.74) | 3.35× (3.19–3.53) | 0.26× (0.25–0.28) | 0.27× (0.26–0.29) | 0.30× (0.28–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.74× (0.73–0.74) | 5.22× (5.21–5.22) | 0.33× (0.33–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.51× (0.49–0.52) | 0.41× (0.39–0.43) | 0.91× (0.89–0.94) | 0.73× (0.71–0.75) | 3.16× (3.07–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.79× (0.75–0.83) | 2.50× (2.44–2.56) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.84) | 5.23× (5.22–5.24) | 0.32× (0.32–0.33) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 43 MB | 0.47× (0.45–0.48) | 0.36× (0.33–0.40) | 0.83× (0.81–0.86) | 0.69× (0.65–0.73) | 2.87× (2.77–2.96) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.27–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.49× (0.46–0.51) | 0.80× (0.78–0.82) | 0.80× (0.78–0.83) | 2.30× (2.26–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.21–0.21) | 1.09× (1.08–1.09) | 0.77× (0.77–0.78) | 5.41× (5.40–5.42) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.62× (1.61–1.63) | 0.77× (0.76–0.77) | 5.36× (5.33–5.38) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.63× (1.62–1.63) | 0.77× (0.77–0.78) | 5.38× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 1 | 18 MB | 0.47× (0.47–0.48) | 0.40× (0.40–0.41) | 1.47× (1.47–1.48) | 0.69× (0.68–0.70) | 6.65× (6.62–6.67) | 0.40× (0.40–0.41) | 0.43× (0.43–0.44) | 0.65× (0.65–0.66) | 0.32× (0.32–0.33) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 2 | 37 MB | 0.45× (0.43–0.46) | 0.39× (0.36–0.42) | 1.07× (1.03–1.11) | 0.67× (0.64–0.70) | 3.32× (3.23–3.42) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.48 MB → 12.3 MB, 5.0×) | 4 | 55 MB | 0.52× (0.50–0.53) | 0.48× (0.44–0.53) | 0.93× (0.92–0.94) | 0.74× (0.71–0.77) | 2.38× (2.34–2.42) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 333 | 3.01 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 570 | 3.26 | 1.08× (1.07–1.09) | 86% | 1.02× (1.01–1.04) | 1.05× (1.03–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 706 | 5.37 | 1.78× (1.75–1.81) | 53% | 1.23× (1.20–1.25) | 1.66× (1.63–1.69) |

**Best:** mibz2 at 4 threads (706 MB/s); lbzip2 at 4 threads (190 MB/s); pbz2 at 4 threads (285 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.07, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.52, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.45. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.92 | 0.02 | 0.09 | 0.297 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2758 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m05s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.96 | 12.3% |
| MTF + RLE2 | 1.35 | 17.3% |
| T-vector build | 1.31 | 16.7% |
| walk (the one each build uses) | 3.73 | 47.7% |
| of which: segment assembly | 0.16 | 2.0% |
| RLE1 | 0.25 | 3.2% |
| CRC | 0.22 | 2.8% |
| **total** | 7.81 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.490 | 1.304 | (in its total) | 2.66× (2.65–2.67) |  |
| walk | 1.247 | 2.660 | (in its total) | 2.14× (2.12–2.15) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.059 | (in its total) | 1.22× (1.19–1.25) |  |
| inverse BWT, total | 1.784 | 4.023 | 3.381 | 2.26× (2.25–2.27) | 1.89× (1.87–1.90) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h31m ± 10m41s |
| 4 threads | 43m22s ± 6m06s |
| 4 threads, the sample's overlap gap as a fixed cost | 33m06s ± 4m41s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 33m06s ± 4m41s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.34 ns a byte of output at 4 threads (CPU, summed over them), 2.50 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,479,937 | 12,349,355 | \`59f0275fb4929505…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2758.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.6.0 (bench/report.py), 10 rounds, threads 1, 2, 4
