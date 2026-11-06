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

The latest record: [`9c7b49c`](<https://github.com/lmmx/mibz2/commit/9c7b49c3f9c8037ca1cf072daf713f4acab915ae>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42137936484>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.537 s | 390 | 6.7 | 2.29× (2.27–2.32) | 5.27× (5.22–5.31) | 5.47× (5.42–5.52) | 6.19× (6.11–6.27) | 5.48× (5.41–5.55) | 6.10× (6.03–6.18) | 5.48× (5.41–5.54) | 1.70× (1.68–1.73) | 1.87× (1.85–1.89) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.313 s | 671 | 7.3 | 2.32× (2.25–2.40) | 4.84× (4.67–5.01) | 5.64× (5.41–5.87) | 5.75× (5.56–5.95) | 5.08× (4.85–5.33) | 10.54× (10.16–10.94) | 9.46× (9.14–9.79) | 2.94× (2.82–3.06) | 3.23× (3.12–3.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.258 s | 812 | 12.2 | 2.35× (2.29–2.41) | 3.53× (3.46–3.60) | 4.41× (4.30–4.52) | 4.14× (4.08–4.21) | 4.13× (4.05–4.20) | 12.72× (12.44–13.02) | 11.41× (11.15–11.68) | 3.54× (3.47–3.62) | 3.89× (3.79–4.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,114 | 2.6 | 0.57× (0.56–0.57) | 3.41× (3.38–3.45) | 4.61× (4.48–4.75) | 3.56× (3.53–3.60) | 6.13× (6.06–6.20) | 2.66× (2.63–2.70) | 2.64× (2.61–2.67) | 0.98× (0.96–1.01) | 1.76× (1.65–1.88) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 s | 1,102 | 2.6 | 0.53× (0.52–0.54) | 2.83× (2.77–2.89) | 5.67× (5.56–5.79) | 3.54× (3.47–3.61) | 5.93× (5.81–6.05) | 2.64× (2.59–2.70) | 2.62× (2.57–2.68) | 0.98× (0.94–1.01) | 1.75× (1.65–1.85) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 s | 1,111 | 2.5 | 0.54× (0.52–0.55) | 2.85× (2.79–2.91) | 5.68× (5.57–5.80) | 3.56× (3.48–3.63) | 6.09× (5.96–6.22) | 2.65× (2.59–2.71) | 2.63× (2.57–2.68) | 0.98× (0.94–1.01) | 1.75× (1.63–1.87) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.029 s | 678 | 4.1 | 2.47× (2.44–2.49) | 2.83× (2.81–2.85) | 3.55× (3.52–3.57) | 3.77× (3.74–3.79) | 4.13× (4.09–4.18) | 3.16× (3.14–3.18) | 2.10× (2.08–2.11) | 1.98× (1.86–2.11) | 2.82× (2.81–2.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,008 | 4.9 | 2.05× (1.87–2.26) | 2.61× (2.45–2.77) | 3.83× (3.54–4.15) | 3.24× (3.02–3.48) | 4.66× (4.37–4.95) | 4.62× (4.35–4.91) | 3.07× (2.90–3.24) | 2.90× (2.63–3.20) | 4.13× (3.88–4.40) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 873 | 10.2 | 1.68× (1.57–1.78) | 1.71× (1.58–1.85) | 2.83× (2.55–3.13) | 1.90× (1.78–2.04) | 3.64× (3.39–3.92) | 3.99× (3.71–4.29) | 2.64× (2.46–2.84) | 2.50× (2.27–2.76) | 3.56× (3.32–3.82) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.111 s | 72 | 36.5 | 1.38× (1.37–1.40) | 2.67× (2.65–2.69) | 2.43× (2.37–2.48) | 2.50× (2.48–2.52) | 2.80× (2.76–2.85) | 3.17× (3.14–3.19) | 3.15× (3.12–3.19) | 1.26× (1.25–1.26) | 1.41× (1.38–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.069 s | 116 | 41.2 | 1.29× (1.27–1.32) | 2.50× (2.42–2.59) | 2.39× (2.34–2.44) | 3.17× (3.09–3.25) | 2.87× (2.82–2.91) | 5.15× (5.06–5.23) | 5.12× (5.06–5.19) | 2.04× (2.00–2.08) | 2.29× (2.24–2.35) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 143 | 54.8 | 1.36× (1.33–1.39) | 2.14× (2.05–2.23) | 2.17× (2.08–2.27) | 2.71× (2.63–2.79) | 2.98× (2.89–3.06) | 6.36× (6.20–6.51) | 6.33× (6.18–6.48) | 2.52× (2.46–2.58) | 2.83× (2.74–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.119 s | 81 | 32.6 | 1.55× (1.52–1.57) | 3.10× (3.06–3.14) | 2.69× (2.63–2.74) | 2.80× (2.78–2.83) | 3.18× (3.05–3.31) | 4.01× (3.99–4.03) | 4.13× (4.09–4.18) | 1.54× (1.51–1.56) | 1.85× (1.75–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.072 s | 133 | 37.0 | 1.37× (1.35–1.39) | 2.81× (2.74–2.89) | 2.58× (2.51–2.65) | 3.59× (3.54–3.64) | 3.12× (3.07–3.16) | 6.57× (6.47–6.66) | 6.77× (6.68–6.86) | 2.51× (2.47–2.56) | 3.04× (2.85–3.24) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 s | 156 | 52.4 | 1.44× (1.39–1.49) | 2.26× (2.18–2.35) | 2.29× (2.22–2.36) | 3.08× (2.98–3.18) | 2.91× (2.82–3.00) | 7.60× (7.40–7.80) | 7.84× (7.60–8.07) | 2.91× (2.83–2.99) | 3.51× (3.29–3.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 837 | 3.5 | 0.62× (0.61–0.62) | 3.43× (3.39–3.47) | 4.39× (4.18–4.62) | 3.12× (2.99–3.26) | 6.65× (6.51–6.79) | 2.32× (2.29–2.35) | 2.30× (2.27–2.33) | 0.95× (0.94–0.96) | 1.73× (1.70–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 794 | 3.8 | 0.49× (0.43–0.55) | 2.34× (2.08–2.63) | 4.69× (4.17–5.28) | 2.79× (2.49–3.14) | 6.04× (5.35–6.83) | 2.09× (1.86–2.36) | 2.08× (1.85–2.34) | 0.86× (0.76–0.97) | 1.56× (1.39–1.75) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 790 | 3.7 | 0.52× (0.51–0.53) | 2.46× (2.42–2.50) | 4.93× (4.84–5.01) | 2.96× (2.90–3.01) | 6.31× (6.18–6.45) | 2.20× (2.16–2.24) | 2.18× (2.14–2.22) | 0.90× (0.89–0.91) | 1.64× (1.61–1.66) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 0.066 s | 189 | 14.1 | 1.44× (1.42–1.46) | 3.10× (3.03–3.16) | 3.39× (3.34–3.44) | 3.55× (3.48–3.63) | 3.76× (3.67–3.85) | 3.81× (3.76–3.85) | 3.69× (3.66–3.72) | 1.18× (1.15–1.20) | 1.38× (1.37–1.40) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 0.043 s | 289 | 16.8 | 1.24× (1.12–1.38) | 2.65× (2.41–2.92) | 3.07× (2.81–3.35) | 3.44× (3.16–3.75) | 3.49× (3.23–3.77) | 5.70× (5.26–6.17) | 5.52× (5.08–6.00) | 1.76× (1.62–1.91) | 2.07× (1.91–2.24) | yes |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 0.036 s | 343 | 25.0 | 1.33× (1.27–1.39) | 2.09× (1.95–2.23) | 2.76× (2.60–2.93) | 3.08× (2.95–3.22) | 3.41× (3.26–3.58) | 6.94× (6.67–7.22) | 6.72× (6.48–6.98) | 2.14× (2.07–2.22) | 2.52× (2.42–2.63) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.29× (2.26–2.32) | 5.24× (5.20–5.29) | 5.44× (5.39–5.50) | 6.17× (6.10–6.25) | 5.46× (5.40–5.53) | 6.09× (6.01–6.16) | 5.46× (5.40–5.52) | 1.70× (1.67–1.72) | 1.85× (1.84–1.87) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.16× (2.08–2.25) | 4.89× (4.70–5.08) | 5.14× (4.94–5.35) | 5.90× (5.69–6.12) | 4.98× (4.75–5.22) | 5.48× (5.31–5.67) | 4.92× (4.77–5.07) | 1.53× (1.47–1.59) | 1.67× (1.62–1.73) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.15× (2.11–2.19) | 3.63× (3.53–3.73) | 3.96× (3.87–4.04) | 4.27× (4.18–4.36) | 3.76× (3.68–3.83) | 3.35× (3.24–3.45) | 3.00× (2.91–3.10) | 0.93× (0.91–0.96) | 1.02× (0.99–1.05) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.60–0.61) | 2.91× (2.88–2.93) | 3.93× (3.84–4.02) | 3.33× (3.30–3.36) | 5.66× (5.60–5.73) | 2.51× (2.48–2.54) | 2.49× (2.47–2.52) | 0.98× (0.96–1.01) | 1.62× (1.60–1.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.59) | 2.73× (2.68–2.78) | 4.93× (4.85–5.00) | 3.29× (3.22–3.35) | 5.42× (5.32–5.52) | 2.46× (2.41–2.50) | 2.44× (2.40–2.49) | 0.96× (0.93–0.99) | 1.58× (1.55–1.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.58× (0.56–0.59) | 3.05× (3.01–3.09) | 5.03× (4.95–5.10) | 3.38× (3.31–3.44) | 5.67× (5.55–5.78) | 2.52× (2.47–2.57) | 2.50× (2.46–2.55) | 0.99× (0.96–1.02) | 1.62× (1.59–1.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.38× (2.36–2.40) | 2.61× (2.60–2.63) | 3.14× (3.11–3.17) | 3.60× (3.58–3.63) | 3.95× (3.91–3.99) | 3.03× (3.02–3.05) | 2.03× (2.02–2.04) | 1.93× (1.82–2.04) | 2.45× (2.42–2.48) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.10× (2.05–2.16) | 2.32× (2.22–2.42) | 3.12× (3.05–3.19) | 3.10× (3.02–3.17) | 3.38× (3.30–3.46) | 2.48× (2.41–2.55) | 1.66× (1.62–1.70) | 1.57× (1.47–1.69) | 2.00× (1.97–2.04) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.80–1.93) | 1.55× (1.47–1.63) | 2.07× (1.92–2.23) | 1.91× (1.81–2.02) | 1.93× (1.83–2.03) | 1.22× (1.16–1.28) | 0.82× (0.78–0.86) | 0.77× (0.72–0.83) | 0.98× (0.94–1.03) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.38× (1.36–1.39) | 2.58× (2.56–2.61) | 2.37× (2.33–2.41) | 2.47× (2.45–2.49) | 2.78× (2.73–2.82) | 3.13× (3.11–3.16) | 3.12× (3.08–3.16) | 1.25× (1.24–1.26) | 1.36× (1.35–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.28) | 2.40× (2.38–2.42) | 2.14× (2.12–2.16) | 2.91× (2.89–2.93) | 2.49× (2.48–2.51) | 2.77× (2.75–2.79) | 2.76× (2.74–2.78) | 1.11× (1.10–1.12) | 1.21× (1.20–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.46–1.50) | 2.24× (2.19–2.30) | 1.91× (1.87–1.95) | 2.63× (2.58–2.67) | 2.51× (2.44–2.58) | 2.09× (2.07–2.12) | 2.08× (2.06–2.11) | 0.84× (0.83–0.84) | 0.91× (0.90–0.92) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.54× (1.51–1.57) | 3.02× (2.99–3.05) | 2.62× (2.56–2.68) | 2.78× (2.75–2.80) | 3.15× (3.02–3.28) | 3.97× (3.94–3.99) | 4.09× (4.04–4.14) | 1.53× (1.50–1.55) | 1.80× (1.69–1.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.39× (1.38–1.40) | 2.76× (2.73–2.80) | 2.39× (2.33–2.45) | 3.34× (3.30–3.39) | 2.78× (2.75–2.81) | 3.48× (3.44–3.52) | 3.59× (3.53–3.64) | 1.34× (1.32–1.36) | 1.57× (1.48–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.51–1.65) | 2.49× (2.41–2.57) | 2.01× (1.95–2.06) | 2.84× (2.76–2.92) | 2.60× (2.56–2.65) | 2.46× (2.42–2.50) | 2.53× (2.48–2.59) | 0.95× (0.92–0.97) | 1.11× (1.05–1.18) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.66–0.67) | 2.43× (2.41–2.46) | 3.45× (3.30–3.60) | 2.85× (2.74–2.97) | 5.93× (5.82–6.05) | 2.15× (2.13–2.17) | 2.14× (2.11–2.16) | 0.96× (0.94–0.98) | 1.36× (1.34–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.56× (0.50–0.62) | 2.13× (1.91–2.37) | 3.80× (3.40–4.25) | 2.59× (2.33–2.88) | 5.37× (4.81–5.99) | 1.92× (1.73–2.14) | 1.91× (1.72–2.13) | 0.86× (0.77–0.96) | 1.21× (1.09–1.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.49× (2.45–2.53) | 4.10× (3.99–4.21) | 2.81× (2.76–2.87) | 5.75× (5.64–5.87) | 2.07× (2.04–2.11) | 2.06× (2.03–2.09) | 0.93× (0.92–0.94) | 1.31× (1.29–1.33) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 1.42× (1.40–1.44) | 2.94× (2.88–3.00) | 3.25× (3.20–3.30) | 3.49× (3.41–3.56) | 3.69× (3.60–3.78) | 3.73× (3.69–3.78) | 3.62× (3.59–3.65) | 1.17× (1.15–1.20) | 1.27× (1.26–1.27) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 1.25× (1.17–1.33) | 2.57× (2.36–2.80) | 2.82× (2.64–3.01) | 3.20× (3.00–3.42) | 3.07× (2.87–3.28) | 3.07× (2.87–3.28) | 2.98× (2.78–3.19) | 0.96× (0.90–1.04) | 1.04× (0.97–1.12) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 1.42× (1.35–1.50) | 2.23× (2.13–2.35) | 2.55× (2.41–2.69) | 2.73× (2.64–2.83) | 2.68× (2.61–2.76) | 2.13× (2.09–2.16) | 2.06× (2.03–2.10) | 0.67× (0.65–0.68) | 0.72× (0.71–0.73) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.37× (0.35–0.38) | 1.08× (1.08–1.09) | 0.86× (0.86–0.86) | 12.24× (12.23–12.26) | 0.28× (0.28–0.28) | 0.31× (0.30–0.31) | 0.47× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 49 MB | 0.38× (0.37–0.38) | 0.48× (0.41–0.55) | 0.79× (0.78–0.81) | 0.66× (0.65–0.68) | 6.34× (6.22–6.45) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.24) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.39–0.39) | 0.63× (0.52–0.76) | 0.71× (0.69–0.72) | 0.62× (0.61–0.63) | 4.12× (4.10–4.14) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.16) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.33× (4.32–4.33) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.20) | 0.16× (0.16–0.16) | 1.80× (1.80–1.81) | 0.84× (0.84–0.84) | 4.46× (4.45–4.46) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.94–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.83–0.84) | 4.45× (4.44–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.45× (0.45–0.45) | 1.74× (1.73–1.74) | 0.66× (0.66–0.67) | 8.18× (8.16–8.21) | 0.65× (0.65–0.65) | 0.69× (0.69–0.69) | 0.75× (0.74–0.75) | 0.38× (0.37–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.63–0.68) | 0.62× (0.56–0.69) | 1.63× (1.56–1.69) | 0.80× (0.76–0.83) | 5.89× (5.65–6.13) | 0.47× (0.45–0.49) | 0.49× (0.47–0.51) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.62× (0.59–0.66) | 0.69× (0.65–0.72) | 1.17× (1.12–1.22) | 0.76× (0.72–0.80) | 3.40× (3.24–3.57) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.28–0.32) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.20× (1.19–1.20) | 0.74× (0.74–0.74) | 5.29× (5.28–5.30) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.46–0.51) | 0.41× (0.39–0.43) | 0.90× (0.87–0.93) | 0.69× (0.65–0.72) | 3.10× (2.99–3.21) | 0.19× (0.18–0.20) | 0.20× (0.19–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.90) | 0.81× (0.79–0.84) | 2.54× (2.48–2.61) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.84× (0.84–0.84) | 5.30× (5.29–5.30) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) | 0.52× (0.52–0.53) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.46× (0.44–0.48) | 0.36× (0.34–0.39) | 0.84× (0.81–0.88) | 0.70× (0.66–0.74) | 2.89× (2.78–3.00) | 0.18× (0.17–0.18) | 0.19× (0.18–0.19) | 0.29× (0.27–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.50× (0.46–0.53) | 0.81× (0.80–0.82) | 0.82× (0.80–0.84) | 2.32× (2.30–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 22 MB | 0.37× (0.37–0.37) | 0.24× (0.22–0.25) | 1.10× (1.09–1.10) | 0.77× (0.77–0.78) | 5.48× (5.47–5.49) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.24) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.77–0.77) | 5.44× (5.41–5.46) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.22–0.23) | 0.21× (0.21–0.21) | 1.65× (1.65–1.66) | 0.77× (0.77–0.78) | 5.46× (5.45–5.48) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.93× (0.92–0.93) | 0.13× (0.13–0.13) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.50× (1.50–1.51) | 0.65× (0.65–0.66) | 6.79× (6.77–6.81) | 0.40× (0.40–0.41) | 0.43× (0.43–0.43) | 0.66× (0.65–0.66) | 0.33× (0.33–0.33) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 2 | 33 MB | 0.48× (0.46–0.49) | 0.42× (0.37–0.47) | 1.17× (1.14–1.21) | 0.70× (0.67–0.73) | 3.65× (3.54–3.76) | 0.22× (0.21–0.23) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.18× (0.17–0.18) |
| `text` (1.81 MB → 12.4 MB, 6.9×) | 4 | 53 MB | 0.51× (0.50–0.52) | 0.50× (0.46–0.54) | 0.95× (0.93–0.97) | 0.75× (0.73–0.78) | 2.43× (2.40–2.46) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 390 | 2.57 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 671 | 2.81 | 1.11× (1.07–1.15) | 86% | 1.03× (1.02–1.05) | 1.05× (1.03–1.07) |
| 4: all 4 CPUs, sharing SMT siblings | 812 | 4.72 | 1.82× (1.76–1.88) | 52% | 1.26× (1.22–1.30) | 1.71× (1.66–1.76) |

**Best:** mibz2 at 4 threads (812 MB/s); lbzip2 at 4 threads (232 MB/s); pbz2 at 4 threads (351 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.01, lbzip2 1t 0.07, lbzip2 2t 0.22, lbzip2 4t 0.40, pbz2 1t 0.00, pbz2 2t 0.36, pbz2 4t 0.46. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.80 | 0.02 | 0.09 | 0.258 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.78 | 12.3% |
| MTF + RLE2 | 1.07 | 16.8% |
| T-vector build | 1.25 | 19.7% |
| walk (the one each build uses) | 2.95 | 46.3% |
| of which: segment assembly | 0.11 | 1.8% |
| RLE1 | 0.14 | 2.2% |
| CRC | 0.18 | 2.8% |
| **total** | 6.36 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.454 | 1.064 | (in its total) | 2.34× (2.31–2.38) |  |
| walk | 1.106 | 2.069 | (in its total) | 1.87× (1.83–1.91) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.043 | 0.045 | (in its total) | 1.07× (1.00–1.13) |  |
| inverse BWT, total | 1.603 | 3.176 | 1.898 | 1.98× (1.94–2.02) | 1.18× (1.16–1.19) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h18m ± 9m22s |
| 4 threads | 37m36s ± 5m22s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m40s ± 4m12s |
| the scanner alone (its serial part) | 11m47s ± 19.7s |
| the workers alone, over 4 | 28m40s ± 4m12s |

At 4 threads the workers bound it. Per byte: the scanner 6.8 ns a byte of bz2, the workers 3.76 ns a byte of output at 4 threads (CPU, summed over them), 2.18 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,807,836 | 12,402,701 | \`8331caec1322f098…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.2 (bench/report.py), 10 rounds, threads 1, 2, 4
