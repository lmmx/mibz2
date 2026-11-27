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

The latest record: [`5767825`](<https://github.com/lmmx/mibz2/commit/576782595c83135d16374208250baf60613e0247>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46059063956>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.382 s | 548 | 4.8 | 3.16× (3.12–3.21) | 7.27× (7.18–7.35) | 7.51× (7.42–7.61) | 8.52× (8.43–8.61) | 7.53× (7.45–7.61) | 7.89× (7.81–7.97) | 7.58× (7.49–7.67) | 2.34× (2.31–2.37) | 2.61× (2.58–2.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.226 s | 929 | 5.1 | 3.21× (3.08–3.34) | 6.54× (6.25–6.84) | 7.64× (7.33–7.97) | 7.79× (7.51–8.08) | 6.89× (6.61–7.17) | 13.51× (12.98–14.07) | 12.97× (12.48–13.48) | 4.00× (3.86–4.15) | 4.47× (4.31–4.64) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.177 s | 1,183 | 8.1 | 3.23× (3.18–3.27) | 4.87× (4.76–4.99) | 5.96× (5.83–6.09) | 5.90× (5.79–6.01) | 5.76× (5.66–5.87) | 17.11× (16.93–17.30) | 16.42× (16.24–16.61) | 5.07× (5.01–5.13) | 5.66× (5.59–5.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,096 | 2.6 | 0.56× (0.55–0.57) | 3.35× (3.29–3.41) | 4.47× (4.39–4.55) | 3.51× (3.45–3.56) | 6.07× (5.95–6.18) | 2.60× (2.55–2.64) | 2.61× (2.56–2.66) | 0.96× (0.94–0.98) | 1.89× (1.71–2.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,050 | 2.8 | 0.50× (0.49–0.51) | 2.67× (2.61–2.73) | 5.35× (5.23–5.47) | 3.36× (3.29–3.43) | 5.70× (5.59–5.81) | 2.48× (2.42–2.54) | 2.49× (2.43–2.55) | 0.91× (0.89–0.93) | 1.80× (1.61–2.02) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,042 | 2.8 | 0.50× (0.49–0.51) | 2.65× (2.60–2.71) | 5.35× (5.18–5.52) | 3.34× (3.27–3.41) | 5.72× (5.61–5.84) | 2.45× (2.41–2.50) | 2.46× (2.41–2.52) | 0.90× (0.88–0.93) | 1.78× (1.59–2.00) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.025 s | 770 | 3.6 | 2.81× (2.79–2.83) | 3.24× (3.15–3.34) | 4.01× (3.99–4.04) | 4.27× (4.24–4.30) | 4.74× (4.69–4.79) | 2.90× (2.87–2.92) | 2.37× (2.35–2.40) | 2.17× (2.15–2.19) | 3.20× (3.17–3.22) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.019 s | 1,040 | 4.3 | 2.20× (2.03–2.38) | 2.82× (2.64–3.02) | 4.25× (3.80–4.76) | 3.62× (3.32–3.94) | 5.08× (4.76–5.42) | 4.04× (3.77–4.33) | 3.31× (3.09–3.55) | 3.03× (2.82–3.25) | 4.46× (4.16–4.79) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 s | 1,051 | 8.6 | 2.04× (1.90–2.20) | 1.83× (1.69–1.97) | 3.23× (2.95–3.54) | 2.48× (2.20–2.79) | 4.56× (4.22–4.93) | 3.94× (3.70–4.20) | 3.23× (3.04–3.44) | 2.96× (2.78–3.14) | 4.35× (4.09–4.63) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.101 s | 80 | 33.2 | 1.53× (1.50–1.56) | 2.92× (2.90–2.95) | 2.62× (2.62–2.63) | 2.75× (2.74–2.77) | 3.07× (3.06–3.09) | 3.28× (3.26–3.31) | 3.43× (3.42–3.44) | 1.37× (1.36–1.37) | 1.52× (1.50–1.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.060 s | 133 | 36.9 | 1.41× (1.34–1.49) | 2.78× (2.66–2.89) | 2.59× (2.47–2.72) | 3.43× (3.22–3.66) | 3.19× (3.04–3.36) | 5.35× (5.07–5.65) | 5.58× (5.30–5.88) | 2.23× (2.11–2.34) | 2.48× (2.35–2.62) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.049 s | 163 | 48.9 | 1.52× (1.47–1.57) | 2.47× (2.36–2.58) | 2.39× (2.31–2.48) | 3.03× (2.90–3.16) | 3.31× (3.24–3.38) | 6.72× (6.59–6.86) | 7.01× (6.88–7.14) | 2.79× (2.74–2.85) | 3.11× (3.03–3.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.110 s | 87 | 30.2 | 1.61× (1.54–1.68) | 3.26× (3.12–3.41) | 2.81× (2.69–2.95) | 2.96× (2.83–3.10) | 3.31× (3.20–3.43) | 3.92× (3.75–4.10) | 4.29× (4.11–4.49) | 1.60× (1.55–1.65) | 1.91× (1.84–1.97) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.066 s | 146 | 34.2 | 1.49× (1.44–1.54) | 3.09× (2.98–3.21) | 2.77× (2.69–2.85) | 3.94× (3.80–4.09) | 3.42× (3.31–3.53) | 6.60× (6.41–6.78) | 7.22× (7.02–7.43) | 2.69× (2.59–2.80) | 3.21× (3.10–3.31) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.056 s | 172 | 47.0 | 1.59× (1.55–1.64) | 2.56× (2.51–2.61) | 2.53× (2.43–2.63) | 3.44× (3.37–3.51) | 3.22× (3.15–3.28) | 7.84× (7.70–7.99) | 8.59× (8.44–8.73) | 3.20× (3.16–3.25) | 3.81× (3.76–3.87) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 833 | 3.5 | 0.62× (0.62–0.63) | 3.44× (3.41–3.48) | 4.32× (4.28–4.36) | 3.08× (3.05–3.11) | 6.78× (6.67–6.90) | 2.28× (2.25–2.31) | 2.32× (2.30–2.34) | 0.95× (0.94–0.96) | 1.73× (1.71–1.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 778 | 4.0 | 0.50× (0.50–0.51) | 2.41× (2.38–2.44) | 4.92× (4.75–5.09) | 2.89× (2.86–2.92) | 6.32× (6.18–6.46) | 2.12× (2.09–2.15) | 2.15× (2.13–2.17) | 0.88× (0.87–0.89) | 1.61× (1.59–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 758 | 4.0 | 0.50× (0.49–0.51) | 2.34× (2.30–2.39) | 4.70× (4.61–4.79) | 2.80× (2.75–2.85) | 6.07× (5.93–6.22) | 2.05× (2.01–2.10) | 2.09× (2.04–2.13) | 0.85× (0.83–0.87) | 1.56× (1.53–1.59) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 0.051 s | 288 | 9.3 | 2.03× (2.01–2.04) | 4.14× (4.13–4.16) | 4.63× (4.53–4.73) | 4.87× (4.84–4.89) | 5.06× (5.04–5.08) | 4.81× (4.77–4.85) | 4.92× (4.90–4.94) | 1.66× (1.65–1.67) | 1.97× (1.96–1.98) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 0.036 s | 414 | 11.9 | 1.65× (1.54–1.77) | 3.44× (3.12–3.80) | 4.07× (3.87–4.27) | 4.66× (4.20–5.18) | 4.47× (4.28–4.67) | 6.83× (6.55–7.11) | 6.99× (6.68–7.30) | 2.35× (2.25–2.46) | 2.79× (2.67–2.92) | yes |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 0.032 s | 467 | 18.1 | 1.57× (1.51–1.63) | 2.47× (2.36–2.59) | 3.47× (3.33–3.62) | 3.76× (3.63–3.89) | 4.15× (4.00–4.31) | 7.76× (7.50–8.03) | 7.94× (7.69–8.20) | 2.68× (2.59–2.77) | 3.18× (3.08–3.28) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.15× (3.11–3.19) | 7.23× (7.15–7.31) | 7.47× (7.38–7.56) | 8.49× (8.39–8.58) | 7.50× (7.42–7.58) | 7.86× (7.78–7.94) | 7.54× (7.46–7.63) | 2.33× (2.30–2.36) | 2.59× (2.56–2.62) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.05× (3.02–3.07) | 6.87× (6.80–6.93) | 7.17× (7.13–7.21) | 8.33× (8.27–8.39) | 7.02× (6.98–7.07) | 7.33× (7.29–7.37) | 7.03× (6.99–7.07) | 2.17× (2.16–2.19) | 2.42× (2.40–2.44) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.06× (3.03–3.08) | 5.12× (5.05–5.19) | 5.50× (5.41–5.59) | 6.21× (6.18–6.25) | 5.33× (5.31–5.35) | 4.66× (4.63–4.69) | 4.47× (4.45–4.50) | 1.38× (1.37–1.39) | 1.54× (1.53–1.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.60× (0.59–0.61) | 2.86× (2.81–2.92) | 3.82× (3.72–3.93) | 3.29× (3.24–3.34) | 5.63× (5.53–5.73) | 2.46× (2.42–2.50) | 2.47× (2.43–2.51) | 0.96× (0.94–0.98) | 1.63× (1.59–1.67) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.54× (0.52–0.55) | 2.54× (2.48–2.59) | 4.57× (4.48–4.65) | 3.07× (3.01–3.13) | 5.13× (5.04–5.22) | 2.27× (2.22–2.32) | 2.28× (2.22–2.33) | 0.89× (0.87–0.91) | 1.50× (1.45–1.56) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.52× (0.52–0.53) | 2.81× (2.75–2.87) | 4.62× (4.48–4.75) | 3.11× (3.05–3.17) | 5.23× (5.14–5.32) | 2.29× (2.25–2.33) | 2.30× (2.25–2.35) | 0.90× (0.88–0.92) | 1.52× (1.47–1.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.69× (2.67–2.71) | 2.97× (2.93–3.02) | 3.49× (3.47–3.51) | 4.07× (4.04–4.09) | 4.51× (4.46–4.55) | 2.77× (2.75–2.79) | 2.29× (2.26–2.31) | 2.10× (2.08–2.12) | 2.74× (2.72–2.77) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.42× (2.39–2.45) | 2.68× (2.63–2.72) | 3.55× (3.49–3.61) | 3.58× (3.54–3.62) | 3.92× (3.88–3.95) | 2.31× (2.28–2.34) | 1.91× (1.89–1.93) | 1.75× (1.73–1.77) | 2.29× (2.26–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.08× (1.97–2.21) | 1.81× (1.73–1.90) | 2.36× (2.26–2.46) | 2.13× (2.03–2.24) | 2.28× (2.18–2.40) | 1.16× (1.11–1.21) | 0.95× (0.91–1.00) | 0.88× (0.84–0.92) | 1.14× (1.10–1.19) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.52× (1.49–1.55) | 2.82× (2.81–2.83) | 2.56× (2.55–2.58) | 2.72× (2.71–2.74) | 3.04× (3.02–3.05) | 3.24× (3.22–3.27) | 3.38× (3.37–3.39) | 1.36× (1.36–1.36) | 1.49× (1.49–1.50) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.41× (1.40–1.42) | 2.69× (2.65–2.74) | 2.37× (2.35–2.39) | 3.22× (3.19–3.25) | 2.80× (2.77–2.82) | 2.91× (2.88–2.95) | 3.04× (3.01–3.06) | 1.22× (1.21–1.23) | 1.34× (1.33–1.35) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.62× (1.57–1.67) | 2.49× (2.41–2.58) | 2.13× (2.08–2.19) | 2.89× (2.83–2.96) | 2.76× (2.70–2.83) | 2.22× (2.17–2.27) | 2.31× (2.26–2.36) | 0.93× (0.91–0.95) | 1.02× (1.00–1.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.60× (1.53–1.67) | 3.16× (3.02–3.30) | 2.75× (2.63–2.87) | 2.93× (2.80–3.07) | 3.28× (3.17–3.39) | 3.88× (3.71–4.05) | 4.25× (4.07–4.43) | 1.59× (1.54–1.64) | 1.84× (1.79–1.89) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.48× (1.44–1.51) | 2.94× (2.87–3.00) | 2.51× (2.45–2.57) | 3.55× (3.47–3.64) | 2.97× (2.90–3.04) | 3.43× (3.36–3.51) | 3.76× (3.68–3.84) | 1.41× (1.37–1.46) | 1.63× (1.58–1.68) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.71× (1.64–1.78) | 2.76× (2.69–2.83) | 2.21× (2.16–2.25) | 3.18× (3.11–3.25) | 2.85× (2.79–2.92) | 2.53× (2.46–2.59) | 2.77× (2.70–2.83) | 1.04× (1.01–1.06) | 1.20× (1.18–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.44× (2.41–2.46) | 3.37× (3.33–3.40) | 2.81× (2.79–2.83) | 6.05× (5.96–6.14) | 2.12× (2.10–2.13) | 2.15× (2.13–2.16) | 0.95× (0.94–0.96) | 1.36× (1.35–1.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.55–0.56) | 2.13× (2.11–2.14) | 3.82× (3.65–3.99) | 2.59× (2.56–2.61) | 5.40× (5.31–5.49) | 1.88× (1.86–1.90) | 1.91× (1.90–1.92) | 0.85× (0.84–0.85) | 1.21× (1.20–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.53× (0.52–0.55) | 2.30× (2.26–2.35) | 3.69× (3.60–3.79) | 2.59× (2.55–2.63) | 5.38× (5.27–5.50) | 1.88× (1.85–1.92) | 1.91× (1.88–1.95) | 0.85× (0.83–0.87) | 1.21× (1.19–1.23) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 2.00× (1.98–2.01) | 3.94× (3.92–3.97) | 4.44× (4.39–4.50) | 4.74× (4.72–4.76) | 4.93× (4.91–4.95) | 4.69× (4.64–4.73) | 4.80× (4.78–4.82) | 1.64× (1.63–1.65) | 1.82× (1.81–1.83) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 1.68× (1.62–1.75) | 3.27× (3.22–3.32) | 3.63× (3.60–3.67) | 4.25× (4.22–4.29) | 3.96× (3.93–4.00) | 3.69× (3.66–3.72) | 3.77× (3.75–3.80) | 1.29× (1.28–1.30) | 1.43× (1.43–1.44) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 1.71× (1.66–1.76) | 2.62× (2.56–2.68) | 2.94× (2.88–3.01) | 3.33× (3.23–3.42) | 3.18× (3.09–3.27) | 2.41× (2.36–2.47) | 2.47× (2.42–2.53) | 0.84× (0.82–0.86) | 0.94× (0.92–0.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.32× (0.31–0.34) | 0.98× (0.98–0.98) | 0.79× (0.78–0.79) | 11.05× (11.04–11.07) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.33–0.36) | 0.50× (0.45–0.56) | 0.71× (0.68–0.75) | 0.61× (0.59–0.63) | 5.71× (5.49–5.94) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.36–0.38) | 0.49× (0.42–0.57) | 0.65× (0.62–0.67) | 0.58× (0.56–0.61) | 3.83× (3.69–3.97) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.21–0.23) | 0.17× (0.17–0.18) | 1.94× (1.89–1.98) | 0.92× (0.90–0.94) | 4.81× (4.69–4.92) | 0.29× (0.28–0.29) | 0.31× (0.30–0.31) | 1.02× (0.99–1.04) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.96–1.96) | 0.93× (0.93–0.94) | 4.86× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.31–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.44× (0.44–0.45) | 1.70× (1.69–1.71) | 0.66× (0.66–0.67) | 8.02× (7.98–8.05) | 0.65× (0.65–0.65) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.69–0.72) | 0.66× (0.62–0.71) | 1.66× (1.63–1.69) | 0.84× (0.82–0.86) | 6.03× (5.92–6.15) | 0.49× (0.48–0.50) | 0.51× (0.51–0.52) | 0.56× (0.55–0.57) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.68× (0.63–0.73) | 0.74× (0.69–0.79) | 1.20× (1.13–1.28) | 0.74× (0.71–0.77) | 3.55× (3.39–3.72) | 0.28× (0.26–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.17× (5.17–5.18) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.51) | 0.41× (0.39–0.43) | 0.90× (0.87–0.93) | 0.72× (0.70–0.75) | 3.12× (3.01–3.22) | 0.19× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.65× (0.63–0.67) | 0.46× (0.45–0.46) | 0.87× (0.86–0.88) | 0.81× (0.79–0.83) | 2.54× (2.50–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.31× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.21× (5.20–5.22) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.47) | 0.37× (0.34–0.39) | 0.84× (0.82–0.85) | 0.71× (0.69–0.73) | 2.88× (2.83–2.92) | 0.18× (0.18–0.18) | 0.19× (0.19–0.19) | 0.29× (0.28–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.62) | 0.50× (0.48–0.52) | 0.80× (0.79–0.81) | 0.82× (0.80–0.84) | 2.31× (2.29–2.34) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.23× (0.22–0.24) | 1.07× (1.06–1.07) | 0.77× (0.76–0.77) | 5.35× (5.33–5.37) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.91) | 6.21× (6.20–6.23) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.28–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.88) | 0.90× (0.90–0.91) | 6.21× (6.19–6.23) | 0.40× (0.40–0.41) | 0.42× (0.41–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.39) | 1.39× (1.39–1.40) | 0.66× (0.65–0.66) | 6.40× (6.37–6.43) | 0.39× (0.39–0.39) | 0.42× (0.42–0.43) | 0.63× (0.62–0.63) | 0.30× (0.30–0.31) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 2 | 36 MB | 0.47× (0.45–0.49) | 0.42× (0.39–0.45) | 1.08× (1.03–1.12) | 0.74× (0.70–0.77) | 3.51× (3.39–3.64) | 0.21× (0.21–0.22) | 0.23× (0.22–0.24) | 0.34× (0.33–0.36) | 0.17× (0.16–0.17) |
| `text` (2.22 MB → 14.8 MB, 6.7×) | 4 | 54 MB | 0.51× (0.50–0.53) | 0.46× (0.42–0.50) | 0.94× (0.92–0.97) | 0.78× (0.75–0.81) | 2.43× (2.38–2.48) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 548 | 1.83 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 929 | 1.98 | 1.07× (1.06–1.09) | 86% | 1.02× (1.01–1.03) | 1.04× (1.02–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 1,183 | 3.12 | 1.69× (1.66–1.71) | 54% | 1.20× (1.17–1.22) | 1.64× (1.62–1.65) |

**Best:** mibz2 at 4 threads (1,183 MB/s); lbzip2 at 4 threads (245 MB/s); pbz2 at 4 threads (368 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.81, mibz2 4t 1.03, lbzip2 1t 0.07, lbzip2 2t 0.26, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.41, pbz2 4t 0.48. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 13%, lbzip2 2t 2%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.51 | 0.01 | 0.08 | 0.177 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 50.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.58 | 9.9% |
| MTF + RLE2 | 1.11 | 18.7% |
| T-vector build | 1.05 | 17.8% |
| walk (the one each build uses) | 2.85 | 48.3% |
| of which: segment assembly | 0.10 | 1.7% |
| RLE1 | 0.13 | 2.3% |
| CRC | 0.18 | 3.0% |
| **total** | 5.91 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.392 | 1.067 | (in its total) | 2.72× (2.71–2.73) |  |
| walk | 1.048 | 2.044 | (in its total) | 1.95× (1.94–1.96) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.036 | 0.041 | (in its total) | 1.13× (1.11–1.14) |  |
| inverse BWT, total | 1.476 | 3.152 | 1.871 | 2.14× (2.12–2.15) | 1.27× (1.26–1.27) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m03s ± 6m43s |
| 4 threads | 25m51s ± 3m38s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m16s ± 2m35s |
| the scanner alone (its serial part) | 8m18s ± 30.2s |
| the workers alone, over 4 | 18m16s ± 2m35s |

At 4 threads the workers bound it. Per byte: the scanner 4.8 ns a byte of bz2, the workers 2.39 ns a byte of output at 4 threads (CPU, summed over them), 1.56 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,218,404 | 14,849,042 | \`9257d589add281a9…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
