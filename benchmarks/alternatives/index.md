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

The latest record: [`70e4cf1`](<https://github.com/lmmx/mibz2/commit/70e4cf1e1535517a5406cd84ea2b3e5a6af9be43>) (2026-11-09).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42539638257>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.370 s | 566 | 4.6 | 2.25× (2.18–2.33) | 5.80× (5.60–6.00) | 6.25× (5.85–6.67) | 6.94× (6.69–7.20) | 6.13× (5.91–6.36) | 6.49× (6.26–6.72) | 5.98× (5.76–6.22) | 1.80× (1.74–1.87) | 2.05× (1.99–2.12) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.218 s | 962 | 5.2 | 2.16× (2.02–2.33) | 4.99× (4.63–5.37) | 5.94× (5.53–6.39) | 6.02× (5.56–6.52) | 5.33× (4.96–5.72) | 10.68× (9.93–11.49) | 9.85× (9.20–10.54) | 2.97× (2.80–3.14) | 3.38× (3.16–3.61) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.193 s | 1,085 | 8.6 | 2.15× (2.07–2.22) | 3.36× (3.23–3.49) | 4.40× (4.18–4.63) | 4.07× (3.92–4.23) | 4.07× (3.93–4.22) | 12.21× (11.69–12.75) | 11.26× (10.80–11.73) | 3.39× (3.26–3.53) | 3.86× (3.70–4.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,434 | 2.0 | 0.57× (0.55–0.58) | 3.65× (3.57–3.73) | 4.59× (4.39–4.79) | 3.77× (3.65–3.89) | 6.57× (6.35–6.79) | 2.93× (2.71–3.17) | 2.83× (2.75–2.90) | 1.04× (0.99–1.08) | 2.19× (2.14–2.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,370 | 2.1 | 0.49× (0.43–0.55) | 2.76× (2.40–3.16) | 5.51× (4.84–6.27) | 3.45× (3.01–3.94) | 6.08× (5.41–6.83) | 2.69× (2.32–3.13) | 2.60× (2.28–2.96) | 0.95× (0.82–1.11) | 2.02× (1.77–2.29) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,394 | 2.0 | 0.52× (0.50–0.53) | 3.01× (2.73–3.32) | 5.77× (5.59–5.96) | 3.66× (3.54–3.78) | 6.41× (6.14–6.68) | 2.82× (2.57–3.09) | 2.72× (2.61–2.83) | 1.00× (0.96–1.03) | 2.11× (2.04–2.18) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 982 | 2.9 | 2.59× (2.48–2.70) | 3.25× (3.06–3.46) | 3.94× (3.71–4.19) | 4.16× (3.96–4.38) | 4.86× (4.68–5.04) | 2.54× (2.42–2.66) | 2.33× (2.23–2.43) | 1.92× (1.84–2.01) | 3.00× (2.87–3.13) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,221 | 3.8 | 1.86× (1.65–2.09) | 2.50× (2.22–2.83) | 3.45× (3.11–3.83) | 3.14× (2.80–3.52) | 4.73× (4.20–5.32) | 3.14× (2.80–3.53) | 2.88× (2.55–3.26) | 2.38× (2.10–2.69) | 3.72× (3.29–4.20) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,104 | 8.1 | 1.62× (1.55–1.69) | 1.76× (1.66–1.87) | 2.65× (2.45–2.86) | 1.99× (1.76–2.25) | 3.89× (3.70–4.08) | 2.86× (2.70–3.03) | 2.62× (2.47–2.78) | 2.17× (2.04–2.30) | 3.38× (3.19–3.58) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.077 s | 103 | 25.6 | 1.45× (1.36–1.54) | 3.17× (3.07–3.26) | 2.91× (2.83–3.00) | 2.82× (2.73–2.92) | 3.15× (3.04–3.27) | 3.72× (3.63–3.82) | 3.61× (3.48–3.73) | 1.37× (1.33–1.42) | 1.55× (1.50–1.60) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.052 s | 155 | 30.8 | 1.30× (1.20–1.40) | 2.83× (2.74–2.91) | 2.65× (2.57–2.74) | 3.48× (3.21–3.78) | 3.11× (2.99–3.23) | 5.72× (5.55–5.90) | 5.54× (5.33–5.77) | 2.11× (2.03–2.20) | 2.38× (2.28–2.48) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 184 | 42.6 | 1.27× (1.24–1.31) | 2.45× (2.32–2.58) | 2.39× (2.31–2.49) | 2.81× (2.66–2.96) | 3.01× (2.93–3.10) | 6.69× (6.47–6.91) | 6.48× (6.19–6.78) | 2.47× (2.39–2.55) | 2.78× (2.70–2.86) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.083 s | 115 | 22.9 | 1.45× (1.26–1.68) | 3.65× (3.23–4.13) | 2.97× (2.65–3.33) | 3.00× (2.65–3.39) | 3.39× (2.96–3.88) | 4.48× (3.98–5.04) | 4.55× (4.04–5.13) | 1.52× (1.35–1.71) | 1.74× (1.53–1.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.055 s | 175 | 27.8 | 1.32× (1.24–1.41) | 3.16× (3.02–3.31) | 2.76× (2.64–2.89) | 3.73× (3.61–3.86) | 3.20× (3.09–3.32) | 7.12× (6.82–7.44) | 7.24× (6.98–7.51) | 2.42× (2.30–2.54) | 2.77× (2.63–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 203 | 39.6 | 1.39× (1.36–1.42) | 2.57× (2.49–2.65) | 2.48× (2.38–2.57) | 3.24× (3.18–3.30) | 3.03× (2.96–3.11) | 8.34× (8.16–8.51) | 8.47× (8.29–8.66) | 2.83× (2.76–2.90) | 3.24× (3.13–3.36) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,064 | 2.9 | 0.60× (0.59–0.61) | 3.22× (3.15–3.29) | 4.48× (4.38–4.58) | 3.10× (3.02–3.18) | 7.02× (6.84–7.20) | 2.32× (2.27–2.37) | 2.32× (2.26–2.37) | 0.97× (0.94–0.99) | 2.15× (2.10–2.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,010 | 3.0 | 0.50× (0.49–0.52) | 3.12× (3.03–3.22) | 5.31× (5.16–5.47) | 3.03× (2.94–3.13) | 6.79× (6.56–7.03) | 2.22× (2.17–2.28) | 2.22× (2.17–2.28) | 0.93× (0.90–0.95) | 2.06× (2.01–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 990 | 3.0 | 0.51× (0.50–0.52) | 3.05× (3.00–3.11) | 5.49× (5.04–5.99) | 3.00× (2.94–3.06) | 6.61× (6.52–6.70) | 2.19× (2.14–2.24) | 2.19× (2.14–2.24) | 0.91× (0.89–0.94) | 2.03× (1.99–2.07) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 0.054 s | 232 | 11.5 | 1.35× (1.23–1.49) | 3.21× (3.10–3.31) | 3.38× (3.27–3.50) | 3.57× (3.40–3.75) | 3.76× (3.67–3.85) | 4.10× (4.01–4.19) | 3.54× (3.47–3.60) | 1.13× (1.10–1.17) | 1.31× (1.25–1.38) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 0.035 s | 354 | 13.9 | 1.10× (1.03–1.18) | 2.60× (2.42–2.80) | 3.11× (2.90–3.33) | 3.40× (3.13–3.70) | 3.61× (3.23–4.05) | 6.09× (5.62–6.61) | 5.26× (4.85–5.70) | 1.69× (1.54–1.84) | 1.95× (1.78–2.14) | yes |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 0.032 s | 396 | 20.9 | 1.15× (1.06–1.24) | 2.13× (1.89–2.40) | 2.64× (2.40–2.91) | 2.82× (2.55–3.11) | 3.19× (2.90–3.51) | 6.86× (6.19–7.61) | 5.92× (5.37–6.53) | 1.90× (1.72–2.09) | 2.20× (1.98–2.44) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.24× (2.17–2.32) | 5.77× (5.58–5.97) | 6.21× (5.82–6.63) | 6.91× (6.66–7.18) | 6.11× (5.89–6.34) | 6.46× (6.24–6.69) | 5.96× (5.74–6.19) | 1.80× (1.74–1.86) | 2.03× (1.97–2.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.04× (1.91–2.17) | 5.04× (4.77–5.33) | 5.41× (5.13–5.70) | 6.23× (5.86–6.61) | 5.26× (4.96–5.58) | 5.64× (5.32–5.97) | 5.20× (4.91–5.51) | 1.57× (1.50–1.64) | 1.77× (1.68–1.87) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.06× (2.00–2.11) | 3.64× (3.49–3.79) | 4.01× (3.82–4.21) | 4.42× (4.24–4.61) | 3.85× (3.72–3.98) | 3.40× (3.27–3.53) | 3.13× (3.02–3.25) | 0.95× (0.92–0.97) | 1.07× (1.03–1.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.63) | 3.11× (3.00–3.23) | 3.97× (3.78–4.18) | 3.49× (3.38–3.61) | 6.02× (5.81–6.23) | 2.74× (2.54–2.95) | 2.64× (2.57–2.72) | 1.04× (0.99–1.09) | 1.83× (1.77–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.46–0.63) | 2.68× (2.30–3.11) | 4.68× (4.05–5.40) | 3.15× (2.72–3.64) | 5.45× (4.79–6.19) | 2.46× (2.10–2.89) | 2.37× (2.06–2.74) | 0.93× (0.79–1.10) | 1.65× (1.43–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.58) | 3.17× (2.93–3.43) | 5.04× (4.86–5.22) | 3.44× (3.34–3.55) | 5.91× (5.68–6.14) | 2.66× (2.43–2.91) | 2.57× (2.47–2.67) | 1.01× (0.97–1.04) | 1.78× (1.71–1.85) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.47× (2.34–2.60) | 2.90× (2.74–3.07) | 3.45× (3.26–3.65) | 3.92× (3.69–4.17) | 4.56× (4.36–4.78) | 2.41× (2.28–2.56) | 2.22× (2.11–2.35) | 1.85× (1.75–1.95) | 2.49× (2.34–2.64) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.06× (1.98–2.14) | 2.36× (2.23–2.51) | 2.83× (2.70–2.98) | 3.17× (2.97–3.39) | 3.66× (3.49–3.83) | 1.83× (1.75–1.92) | 1.69× (1.59–1.79) | 1.40× (1.33–1.48) | 1.89× (1.79–1.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.76× (1.68–1.84) | 1.53× (1.47–1.59) | 1.87× (1.80–1.95) | 1.85× (1.80–1.89) | 2.01× (1.93–2.09) | 0.88× (0.85–0.92) | 0.81× (0.77–0.86) | 0.68× (0.64–0.71) | 0.91× (0.87–0.96) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.44× (1.35–1.53) | 3.08× (2.99–3.17) | 2.81× (2.72–2.89) | 2.79× (2.69–2.88) | 3.11× (3.00–3.22) | 3.67× (3.58–3.76) | 3.56× (3.43–3.68) | 1.37× (1.32–1.42) | 1.47× (1.44–1.51) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.32× (1.20–1.44) | 2.72× (2.65–2.79) | 2.38× (2.33–2.43) | 3.26× (3.04–3.49) | 2.72× (2.64–2.80) | 3.12× (3.03–3.21) | 3.03× (2.94–3.11) | 1.16× (1.12–1.20) | 1.25× (1.22–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.35–1.46) | 2.57× (2.51–2.63) | 2.17× (2.09–2.25) | 2.81× (2.74–2.88) | 2.61× (2.54–2.69) | 2.30× (2.20–2.41) | 2.23× (2.14–2.33) | 0.86× (0.82–0.90) | 0.93× (0.88–0.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.26–1.67) | 3.56× (3.16–4.01) | 2.90× (2.57–3.26) | 2.96× (2.62–3.35) | 3.35× (2.93–3.83) | 4.42× (3.93–4.97) | 4.49× (3.99–5.06) | 1.51× (1.34–1.70) | 1.67× (1.48–1.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.31–1.43) | 3.17× (3.12–3.22) | 2.56× (2.51–2.61) | 3.57× (3.50–3.64) | 2.92× (2.87–2.97) | 3.88× (3.81–3.95) | 3.94× (3.85–4.04) | 1.33× (1.29–1.36) | 1.47× (1.42–1.51) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.54–1.62) | 2.87× (2.81–2.94) | 2.14× (2.08–2.21) | 3.02× (2.96–3.09) | 2.71× (2.64–2.79) | 2.74× (2.66–2.82) | 2.79× (2.69–2.89) | 0.94× (0.91–0.96) | 1.04× (1.01–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.55× (2.46–2.65) | 3.32× (3.23–3.42) | 2.81× (2.74–2.89) | 6.18× (6.02–6.34) | 2.14× (2.09–2.18) | 2.14× (2.08–2.19) | 0.98× (0.96–1.00) | 1.47× (1.43–1.51) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.60) | 2.41× (2.20–2.63) | 3.96× (3.83–4.11) | 2.75× (2.67–2.83) | 5.86× (5.68–6.06) | 2.00× (1.97–2.04) | 2.00× (1.97–2.04) | 0.92× (0.90–0.94) | 1.38× (1.35–1.41) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.56–0.59) | 2.48× (2.41–2.55) | 4.15× (3.79–4.55) | 2.83× (2.78–2.89) | 5.95× (5.86–6.04) | 2.06× (2.01–2.11) | 2.06× (2.01–2.11) | 0.94× (0.92–0.97) | 1.42× (1.37–1.46) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 1.34× (1.22–1.48) | 3.00× (2.90–3.10) | 3.23× (3.13–3.32) | 3.50× (3.33–3.68) | 3.68× (3.59–3.77) | 4.01× (3.92–4.10) | 3.46× (3.40–3.53) | 1.14× (1.10–1.18) | 1.21× (1.15–1.26) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 1.14× (1.07–1.22) | 2.53× (2.31–2.77) | 2.78× (2.59–2.98) | 3.17× (2.91–3.45) | 3.10× (2.75–3.49) | 3.23× (2.98–3.51) | 2.79× (2.57–3.04) | 0.92× (0.84–1.01) | 0.97× (0.89–1.06) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 1.26× (1.20–1.33) | 2.10× (1.92–2.29) | 2.36× (2.20–2.53) | 2.59× (2.41–2.78) | 2.54× (2.35–2.74) | 2.19× (2.03–2.36) | 1.89× (1.75–2.03) | 0.62× (0.57–0.67) | 0.66× (0.61–0.71) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.33) | 0.36× (0.35–0.38) | 1.08× (1.08–1.08) | 0.88× (0.87–0.89) | 12.18× (12.17–12.20) | 0.28× (0.28–0.28) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.38–0.38) | 0.55× (0.48–0.63) | 0.80× (0.79–0.80) | 0.67× (0.66–0.67) | 6.38× (6.37–6.40) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.39× (0.39–0.39) | 0.62× (0.52–0.74) | 0.70× (0.69–0.71) | 0.62× (0.60–0.63) | 4.07× (4.05–4.09) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.26–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.31× (4.31–4.32) | 0.25× (0.25–0.25) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.83–0.84) | 4.43× (4.42–4.44) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.79× (1.79–1.79) | 0.84× (0.84–0.84) | 4.42× (4.42–4.43) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.50–0.51) | 0.46× (0.44–0.47) | 1.73× (1.73–1.73) | 0.67× (0.66–0.68) | 8.15× (8.13–8.17) | 0.64× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.63–0.71) | 0.67× (0.62–0.73) | 1.62× (1.55–1.68) | 0.80× (0.78–0.83) | 5.86× (5.65–6.08) | 0.46× (0.45–0.48) | 0.49× (0.47–0.51) | 0.53× (0.51–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.62× (0.60–0.65) | 0.72× (0.67–0.78) | 1.16× (1.11–1.21) | 0.75× (0.73–0.77) | 3.36× (3.25–3.48) | 0.26× (0.25–0.27) | 0.27× (0.26–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.74× (0.74–0.74) | 5.26× (5.25–5.27) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.49–0.52) | 0.44× (0.42–0.45) | 0.93× (0.90–0.97) | 0.74× (0.70–0.78) | 3.23× (3.11–3.36) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.63–0.66) | 0.46× (0.45–0.47) | 0.87× (0.85–0.90) | 0.81× (0.79–0.84) | 2.54× (2.47–2.61) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.19× (1.19–1.19) | 0.83× (0.83–0.84) | 5.27× (5.26–5.28) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.85× (0.82–0.88) | 0.71× (0.69–0.73) | 2.91× (2.81–3.01) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.48× (0.46–0.51) | 0.80× (0.79–0.82) | 0.82× (0.80–0.85) | 2.31× (2.27–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.36–0.37) | 0.22× (0.21–0.24) | 1.09× (1.09–1.09) | 0.77× (0.77–0.77) | 5.44× (5.44–5.45) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.63× (1.63–1.64) | 0.77× (0.77–0.78) | 5.41× (5.39–5.43) | 0.34× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.20–0.21) | 1.64× (1.63–1.64) | 0.77× (0.77–0.77) | 5.42× (5.41–5.43) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 1 | 18 MB | 0.46× (0.46–0.47) | 0.41× (0.41–0.41) | 1.50× (1.49–1.50) | 0.67× (0.67–0.67) | 6.77× (6.76–6.78) | 0.40× (0.40–0.40) | 0.43× (0.42–0.43) | 0.66× (0.66–0.66) | 0.33× (0.33–0.33) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 2 | 33 MB | 0.49× (0.47–0.50) | 0.43× (0.38–0.47) | 1.17× (1.13–1.21) | 0.73× (0.71–0.76) | 3.68× (3.56–3.81) | 0.22× (0.21–0.23) | 0.23× (0.23–0.24) | 0.36× (0.35–0.37) | 0.18× (0.17–0.19) |
| `text` (2.09 MB → 12.5 MB, 6.0×) | 4 | 52 MB | 0.53× (0.51–0.55) | 0.48× (0.44–0.54) | 0.99× (0.96–1.01) | 0.79× (0.76–0.81) | 2.52× (2.47–2.57) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 566 | 1.77 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 962 | 1.99 | 1.15× (1.07–1.23) | 82% | 1.00× (0.98–1.03) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,085 | 3.32 | 1.90× (1.79–2.02) | 47% | 1.20× (1.17–1.22) | 1.74× (1.69–1.79) |

**Best:** mibz2 at 4 threads (1,085 MB/s); lbzip2 at 4 threads (317 MB/s); pbz2 at 4 threads (494 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.19, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.41, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.50. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.54 | 0.01 | 0.09 | 0.193 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.6s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.61 | 14.3% |
| MTF + RLE2 | 0.88 | 20.6% |
| T-vector build | 0.83 | 19.4% |
| walk (the one each build uses) | 1.71 | 40.0% |
| of which: segment assembly | 0.07 | 1.6% |
| RLE1 | 0.10 | 2.2% |
| CRC | 0.15 | 3.6% |
| **total** | 4.28 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.326 | 0.702 | (in its total) | 2.15× (2.11–2.20) |  |
| walk | 0.659 | 1.595 | (in its total) | 2.43× (2.39–2.47) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.028 | 0.030 | (in its total) | 1.05× (1.00–1.10) |  |
| inverse BWT, total | 1.011 | 2.328 | 1.315 | 2.30× (2.26–2.34) | 1.30× (1.27–1.33) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 54m07s ± 6m37s |
| 4 threads | 28m46s ± 4m12s |
| 4 threads, the sample's overlap gap as a fixed cost | 19m52s ± 2m56s |
| the scanner alone (its serial part) | 9m10s ± 30.2s |
| the workers alone, over 4 | 19m52s ± 2m56s |

At 4 threads the workers bound it. Per byte: the scanner 5.3 ns a byte of bz2, the workers 2.60 ns a byte of output at 4 threads (CPU, summed over them), 1.47 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,094,880 | 12,489,382 | \`0cafc9c0ea86fcad…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
