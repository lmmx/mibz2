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

The latest record: [`5fbf69e`](<https://github.com/lmmx/mibz2/commit/5fbf69e07cacb11518f8f25abe344a3485db9824>) (2026-11-21).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44907108302>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.344 s | 609 | 6.3 | 2.87× (2.84–2.90) | 6.99× (6.89–7.09) | 7.36× (7.23–7.49) | 8.31× (8.19–8.43) | 7.41× (7.27–7.56) | 7.65× (7.50–7.80) | 7.32× (7.13–7.51) | 2.20× (2.14–2.26) | 2.49× (2.46–2.52) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.213 s | 985 | 7.2 | 2.85× (2.67–3.04) | 6.03× (5.79–6.29) | 7.22× (6.82–7.64) | 7.19× (6.89–7.51) | 6.57× (6.25–6.90) | 12.35× (11.80–12.92) | 11.81× (11.24–12.41) | 3.54× (3.37–3.72) | 4.01× (3.82–4.22) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.177 s | 1,183 | 10.9 | 2.86× (2.74–2.99) | 4.56× (4.35–4.79) | 6.18× (5.78–6.61) | 5.63× (5.36–5.92) | 5.69× (5.37–6.03) | 15.11× (14.68–15.54) | 14.45× (14.04–14.86) | 4.34× (4.20–4.47) | 4.91× (4.78–5.05) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,266 | 3.4 | 0.58× (0.57–0.59) | 3.85× (3.76–3.94) | 4.53× (4.42–4.64) | 3.78× (3.70–3.86) | 6.63× (6.51–6.75) | 2.86× (2.80–2.92) | 2.87× (2.82–2.92) | 1.03× (1.02–1.05) | 2.29× (2.09–2.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 s | 1,190 | 3.6 | 0.51× (0.50–0.53) | 3.08× (2.99–3.17) | 5.63× (5.45–5.82) | 3.61× (3.49–3.74) | 6.31× (6.10–6.53) | 2.73× (2.65–2.81) | 2.74× (2.65–2.83) | 0.99× (0.95–1.02) | 2.19× (1.97–2.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 s | 1,170 | 3.6 | 0.51× (0.51–0.52) | 3.04× (2.99–3.10) | 5.67× (5.48–5.86) | 3.59× (3.54–3.65) | 6.28× (6.13–6.42) | 2.69× (2.65–2.73) | 2.70× (2.66–2.73) | 0.97× (0.94–1.01) | 2.15× (1.94–2.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.021 s | 933 | 4.4 | 3.00× (2.95–3.05) | 3.48× (3.38–3.58) | 4.38× (4.27–4.48) | 4.63× (4.55–4.70) | 5.39× (5.31–5.46) | 3.06× (2.97–3.15) | 2.69× (2.63–2.74) | 2.17× (2.15–2.19) | 3.22× (3.10–3.35) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.016 s | 1,252 | 6.0 | 2.23× (2.09–2.39) | 3.05× (2.81–3.30) | 4.08× (3.68–4.53) | 3.66× (3.40–3.93) | 5.55× (5.12–6.02) | 4.08× (3.71–4.48) | 3.58× (3.32–3.86) | 2.89× (2.69–3.12) | 4.30× (3.93–4.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,026 | 12.2 | 1.70× (1.59–1.82) | 1.79× (1.58–2.04) | 2.78× (2.52–3.06) | 2.05× (1.94–2.17) | 4.12× (3.87–4.38) | 3.35× (3.11–3.62) | 2.94× (2.75–3.15) | 2.38× (2.23–2.53) | 3.53× (3.27–3.81) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.097 s | 83 | 47.1 | 1.41× (1.38–1.43) | 2.99× (2.91–3.07) | 2.69× (2.61–2.77) | 2.62× (2.53–2.71) | 3.00× (2.92–3.07) | 3.40× (3.33–3.47) | 3.34× (3.26–3.43) | 1.31× (1.28–1.34) | 1.48× (1.44–1.53) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.062 s | 129 | 53.9 | 1.33× (1.28–1.37) | 2.79× (2.71–2.88) | 2.65× (2.52–2.78) | 3.27× (3.07–3.47) | 3.03× (2.96–3.09) | 5.32× (5.16–5.50) | 5.23× (5.04–5.41) | 2.04× (1.97–2.12) | 2.32× (2.25–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 163 | 69.9 | 1.28× (1.25–1.32) | 2.43× (2.38–2.49) | 2.46× (2.31–2.61) | 2.88× (2.75–3.02) | 3.10× (3.06–3.14) | 6.59× (6.44–6.75) | 6.47× (6.30–6.65) | 2.53× (2.47–2.60) | 2.87× (2.78–2.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.099 s | 97 | 40.2 | 1.51× (1.49–1.53) | 3.62× (3.52–3.72) | 3.00× (2.90–3.10) | 3.00× (2.90–3.10) | 3.43× (3.31–3.55) | 4.32× (4.17–4.48) | 4.54× (4.43–4.67) | 1.58× (1.55–1.61) | 1.82× (1.74–1.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.063 s | 152 | 46.9 | 1.34× (1.28–1.39) | 3.15× (3.06–3.24) | 2.76× (2.66–2.85) | 3.70× (3.56–3.85) | 3.28× (3.17–3.38) | 6.71× (6.51–6.92) | 7.06× (6.83–7.29) | 2.45× (2.36–2.55) | 2.83× (2.69–2.98) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 173 | 67.5 | 1.33× (1.28–1.38) | 2.57× (2.47–2.68) | 2.48× (2.33–2.64) | 3.24× (3.13–3.36) | 3.05× (2.96–3.15) | 7.80× (7.57–8.04) | 8.20× (7.89–8.53) | 2.85× (2.74–2.96) | 3.29× (3.18–3.40) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 930 | 4.8 | 0.63× (0.61–0.65) | 3.03× (2.78–3.30) | 3.89× (3.81–3.96) | 3.16× (3.10–3.22) | 7.05× (6.91–7.20) | 2.36× (2.31–2.41) | 2.37× (2.32–2.43) | 0.98× (0.96–1.01) | 1.91× (1.88–1.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 877 | 5.1 | 0.51× (0.50–0.53) | 2.82× (2.62–3.02) | 5.12× (4.81–5.44) | 3.10× (2.94–3.27) | 6.72× (6.56–6.89) | 2.25× (2.20–2.31) | 2.26× (2.21–2.32) | 0.94× (0.91–0.96) | 1.82× (1.78–1.87) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 862 | 5.1 | 0.51× (0.50–0.52) | 2.67× (2.60–2.74) | 4.93× (4.65–5.23) | 2.99× (2.92–3.05) | 6.59× (6.43–6.76) | 2.20× (2.14–2.25) | 2.21× (2.15–2.26) | 0.91× (0.90–0.93) | 1.78× (1.73–1.82) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.050 s | 266 | 14.9 | 1.79× (1.77–1.81) | 4.25× (4.21–4.30) | 4.61× (4.55–4.67) | 4.74× (4.69–4.79) | 5.01× (4.98–5.05) | 5.06× (4.98–5.13) | 4.80× (4.75–4.86) | 1.57× (1.49–1.66) | 1.81× (1.78–1.84) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.035 s | 384 | 19.1 | 1.53× (1.46–1.61) | 3.41× (3.23–3.61) | 4.36× (4.13–4.60) | 4.51× (4.47–4.55) | 4.71× (4.45–4.98) | 7.22× (6.96–7.50) | 6.86× (6.66–7.08) | 2.25× (2.16–2.34) | 2.58× (2.50–2.68) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.034 s | 393 | 30.3 | 1.39× (1.29–1.51) | 2.54× (2.35–2.76) | 3.23× (3.08–3.38) | 3.32× (3.17–3.47) | 3.90× (3.65–4.17) | 7.48× (7.01–7.99) | 7.11× (6.68–7.57) | 2.33× (2.16–2.51) | 2.68× (2.52–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.86× (2.83–2.89) | 6.94× (6.84–7.04) | 7.31× (7.18–7.44) | 8.27× (8.16–8.39) | 7.38× (7.24–7.52) | 7.61× (7.46–7.77) | 7.28× (7.09–7.48) | 2.19× (2.13–2.25) | 2.47× (2.43–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.68× (2.58–2.78) | 6.31× (6.16–6.46) | 6.83× (6.59–7.08) | 7.70× (7.45–7.96) | 6.64× (6.40–6.90) | 6.66× (6.44–6.90) | 6.37× (6.13–6.63) | 1.92× (1.84–1.99) | 2.16× (2.08–2.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.87× (2.75–2.99) | 5.20× (5.00–5.41) | 6.08× (5.70–6.47) | 6.45× (6.18–6.72) | 5.71× (5.41–6.02) | 4.44× (4.33–4.56) | 4.25× (4.13–4.37) | 1.28× (1.24–1.31) | 1.44× (1.40–1.47) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.12× (3.06–3.18) | 4.01× (3.92–4.10) | 3.50× (3.42–3.57) | 6.06× (5.95–6.18) | 2.68× (2.62–2.73) | 2.68× (2.63–2.73) | 1.03× (1.02–1.05) | 1.86× (1.82–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.59) | 2.79× (2.71–2.88) | 4.93× (4.80–5.06) | 3.33× (3.22–3.44) | 5.70× (5.52–5.89) | 2.52× (2.44–2.59) | 2.52× (2.44–2.61) | 0.97× (0.94–1.01) | 1.75× (1.69–1.81) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.55–0.57) | 3.02× (2.97–3.07) | 5.02× (4.93–5.11) | 3.39× (3.34–3.44) | 5.79× (5.66–5.92) | 2.55× (2.52–2.58) | 2.55× (2.52–2.59) | 0.98× (0.95–1.02) | 1.76× (1.73–1.80) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.84× (2.79–2.89) | 3.16× (3.12–3.21) | 3.84× (3.77–3.90) | 4.34× (4.28–4.41) | 5.04× (4.97–5.12) | 2.90× (2.81–2.99) | 2.56× (2.51–2.60) | 2.08× (2.06–2.10) | 2.78× (2.75–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.21–2.34) | 2.61× (2.48–2.74) | 3.07× (2.94–3.20) | 3.36× (3.22–3.51) | 3.82× (3.65–4.01) | 2.11× (1.98–2.26) | 1.86× (1.78–1.95) | 1.52× (1.46–1.58) | 2.03× (1.94–2.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.90× (1.82–1.98) | 1.64× (1.55–1.74) | 2.01× (1.92–2.10) | 2.04× (1.93–2.15) | 2.17× (2.06–2.29) | 1.06× (0.98–1.14) | 0.93× (0.88–0.98) | 0.76× (0.72–0.80) | 1.01× (0.96–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.40× (1.38–1.42) | 2.90× (2.83–2.98) | 2.61× (2.53–2.69) | 2.59× (2.50–2.68) | 2.96× (2.88–3.03) | 3.36× (3.29–3.43) | 3.30× (3.22–3.38) | 1.30× (1.27–1.33) | 1.41× (1.39–1.44) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.31× (1.29–1.32) | 2.76× (2.70–2.81) | 2.39× (2.31–2.47) | 3.07× (3.00–3.15) | 2.68× (2.63–2.73) | 2.93× (2.87–3.00) | 2.88× (2.80–2.96) | 1.14× (1.11–1.16) | 1.24× (1.21–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.37–1.45) | 2.63× (2.54–2.73) | 2.18× (2.06–2.30) | 2.90× (2.78–3.02) | 2.63× (2.58–2.68) | 2.23× (2.18–2.27) | 2.19× (2.13–2.24) | 0.86× (0.84–0.89) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.50× (1.48–1.52) | 3.54× (3.44–3.64) | 2.90× (2.81–3.00) | 2.96× (2.86–3.06) | 3.38× (3.26–3.50) | 4.26× (4.11–4.41) | 4.48× (4.36–4.60) | 1.57× (1.54–1.60) | 1.76× (1.68–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.37× (1.35–1.39) | 3.21× (3.15–3.27) | 2.56× (2.51–2.62) | 3.57× (3.48–3.67) | 2.98× (2.91–3.04) | 3.65× (3.53–3.77) | 3.84× (3.75–3.93) | 1.34× (1.31–1.38) | 1.50× (1.44–1.57) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.51× (1.46–1.55) | 2.89× (2.79–2.98) | 2.19× (2.06–2.32) | 3.07× (2.99–3.14) | 2.76× (2.67–2.85) | 2.57× (2.48–2.65) | 2.70× (2.63–2.77) | 0.94× (0.92–0.97) | 1.06× (1.02–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.67–0.70) | 2.56× (2.50–2.61) | 3.34× (3.26–3.43) | 2.86× (2.81–2.91) | 6.21× (6.07–6.36) | 2.17× (2.13–2.22) | 2.19× (2.14–2.24) | 0.99× (0.97–1.01) | 1.49× (1.46–1.53) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.39× (2.29–2.50) | 4.09× (3.99–4.20) | 2.84× (2.69–3.00) | 5.85× (5.74–5.96) | 2.04× (1.99–2.08) | 2.05× (2.01–2.09) | 0.93× (0.91–0.94) | 1.40× (1.36–1.44) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.49× (2.45–2.53) | 4.02× (3.96–4.09) | 2.82× (2.77–2.87) | 5.93× (5.79–6.06) | 2.06× (2.01–2.10) | 2.07× (2.03–2.11) | 0.94× (0.92–0.95) | 1.41× (1.39–1.44) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.76× (1.75–1.78) | 4.05× (4.02–4.09) | 4.42× (4.36–4.47) | 4.61× (4.56–4.67) | 4.88× (4.84–4.92) | 4.92× (4.84–5.00) | 4.67× (4.61–4.74) | 1.55× (1.47–1.64) | 1.64× (1.62–1.66) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.49× (1.46–1.51) | 3.30× (3.18–3.41) | 3.76× (3.61–3.91) | 4.19× (4.05–4.34) | 4.01× (3.75–4.28) | 3.82× (3.68–3.96) | 3.63× (3.52–3.74) | 1.21× (1.16–1.26) | 1.27× (1.23–1.32) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.54× (1.47–1.62) | 2.75× (2.50–3.03) | 3.01× (2.84–3.18) | 3.31× (3.20–3.42) | 3.12× (2.99–3.25) | 2.43× (2.36–2.49) | 2.31× (2.25–2.37) | 0.77× (0.72–0.82) | 0.81× (0.79–0.83) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.34× (0.33–0.35) | 0.97× (0.97–0.97) | 0.78× (0.77–0.79) | 10.96× (10.93–10.99) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.34) | 0.43× (0.38–0.50) | 0.66× (0.64–0.68) | 0.56× (0.55–0.58) | 5.34× (5.20–5.49) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 95 MB | 0.33× (0.31–0.34) | 0.50× (0.40–0.62) | 0.59× (0.56–0.61) | 0.51× (0.48–0.54) | 3.39× (3.23–3.55) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.12–0.14) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.81) | 4.28× (4.27–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.76× (1.76–1.77) | 0.83× (0.83–0.83) | 4.37× (4.36–4.38) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.83× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.69) | 0.65× (0.65–0.66) | 7.96× (7.94–7.99) | 0.65× (0.64–0.65) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.64–0.70) | 0.68× (0.62–0.74) | 1.61× (1.55–1.67) | 0.80× (0.78–0.83) | 5.82× (5.60–6.04) | 0.47× (0.45–0.49) | 0.50× (0.48–0.52) | 0.54× (0.52–0.56) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.66× (0.62–0.71) | 1.13× (1.10–1.17) | 0.69× (0.67–0.72) | 3.34× (3.24–3.44) | 0.26× (0.25–0.27) | 0.28× (0.27–0.28) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.16–1.17) | 0.73× (0.72–0.73) | 5.16× (5.14–5.18) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.41× (0.39–0.43) | 0.90× (0.87–0.93) | 0.71× (0.66–0.75) | 3.11× (3.01–3.22) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.66) | 0.46× (0.45–0.47) | 0.88× (0.86–0.89) | 0.82× (0.80–0.83) | 2.54× (2.50–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.16–1.17) | 0.82× (0.82–0.82) | 5.17× (5.15–5.18) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.37× (0.34–0.39) | 0.83× (0.81–0.85) | 0.70× (0.68–0.73) | 2.85× (2.78–2.93) | 0.18× (0.17–0.18) | 0.19× (0.18–0.20) | 0.28× (0.28–0.29) | 0.14× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.59× (0.57–0.60) | 0.48× (0.45–0.51) | 0.79× (0.78–0.81) | 0.82× (0.80–0.84) | 2.28× (2.24–2.32) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.06–1.07) | 0.76× (0.76–0.76) | 5.34× (5.32–5.35) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.76× (0.76–0.77) | 5.27× (5.25–5.29) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.90× (0.90–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 24 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.76× (0.76–0.77) | 5.29× (5.28–5.30) | 0.34× (0.34–0.34) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 20 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.38) | 1.39× (1.39–1.40) | 0.67× (0.66–0.67) | 6.33× (6.32–6.35) | 0.39× (0.39–0.39) | 0.42× (0.41–0.42) | 0.62× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.40× (0.37–0.43) | 1.04× (1.02–1.06) | 0.66× (0.64–0.67) | 3.32× (3.28–3.37) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.33) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 56 MB | 0.50× (0.48–0.51) | 0.46× (0.42–0.51) | 0.89× (0.87–0.92) | 0.71× (0.68–0.75) | 2.30× (2.24–2.35) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 609 | 1.65 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 985 | 1.89 | 1.14× (1.10–1.19) | 81% | 1.04× (1.01–1.07) | 1.07× (1.03–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 1,183 | 2.85 | 1.71× (1.67–1.76) | 49% | 1.28× (1.22–1.35) | 1.72× (1.63–1.81) |

**Best:** mibz2 at 4 threads (1,183 MB/s); lbzip2 at 4 threads (271 MB/s); pbz2 at 4 threads (405 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.86, mibz2 4t 1.26, lbzip2 1t 0.08, lbzip2 2t 0.27, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.49, pbz2 4t 0.51. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 12%, mibz2 4t 13%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.07 | 0.44 | 0.01 | 0.07 | 0.177 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3820 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 44.3s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.03 | 14.3% |
| MTF + RLE2 | 1.45 | 20.1% |
| T-vector build | 1.32 | 18.4% |
| walk (the one each build uses) | 3.01 | 41.9% |
| of which: segment assembly | 0.13 | 1.8% |
| RLE1 | 0.15 | 2.1% |
| CRC | 0.26 | 3.6% |
| **total** | 7.18 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.341 | 0.816 | (in its total) | 2.39× (2.38–2.41) |  |
| walk | 0.750 | 1.827 | (in its total) | 2.44× (2.41–2.46) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.035 | 0.037 | (in its total) | 1.05× (1.01–1.08) |  |
| inverse BWT, total | 1.128 | 2.683 | 1.517 | 2.38× (2.36–2.40) | 1.34× (1.33–1.36) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 50m21s ± 5m49s |
| 4 threads | 25m30s ± 3m39s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m58s ± 2m18s |
| the scanner alone (its serial part) | 9m27s ± 32.9s |
| the workers alone, over 4 | 15m58s ± 2m18s |

At 4 threads the workers bound it. Per byte: the scanner 5.5 ns a byte of bz2, the workers 2.09 ns a byte of output at 4 threads (CPU, summed over them), 1.34 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,147 | 13,382,859 | \`f7d4e4f1aa08d971…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 3820.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
