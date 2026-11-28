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

The latest record: [`ca6366b`](<https://github.com/lmmx/mibz2/commit/ca6366bf06ddd4a05f10aa2bfaa2f0818a840c5b>) (2026-11-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46087902655>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.283 s | 742 | 6.1 | 2.92× (2.87–2.98) | 7.29× (7.21–7.38) | 7.61× (7.53–7.68) | 8.73× (8.63–8.82) | 7.69× (7.62–7.77) | 7.95× (7.86–8.04) | 7.58× (7.49–7.67) | 2.30× (2.25–2.35) | 2.68× (2.62–2.73) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.161 s | 1,303 | 6.6 | 3.01× (2.92–3.09) | 6.83× (6.51–7.17) | 8.22× (7.90–8.56) | 8.22× (7.93–8.52) | 7.27× (7.05–7.50) | 13.91× (13.36–14.49) | 13.26× (12.78–13.76) | 4.03× (3.86–4.21) | 4.69× (4.49–4.90) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.133 s | 1,575 | 10.2 | 3.02× (2.95–3.10) | 4.81× (4.64–4.99) | 6.17× (6.02–6.33) | 5.74× (5.57–5.92) | 5.84× (5.71–5.98) | 16.84× (16.41–17.29) | 16.05× (15.52–16.60) | 4.88× (4.72–5.04) | 5.67× (5.53–5.82) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,444 | 3.4 | 0.57× (0.56–0.59) | 3.68× (3.59–3.77) | 4.62× (4.49–4.76) | 3.76× (3.69–3.83) | 6.70× (6.53–6.87) | 2.93× (2.69–3.18) | 2.84× (2.77–2.91) | 1.01× (0.98–1.03) | 2.21× (2.16–2.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,347 | 3.9 | 0.49× (0.48–0.50) | 2.96× (2.60–3.37) | 5.66× (5.51–5.81) | 3.51× (3.42–3.60) | 6.07× (5.92–6.23) | 2.71× (2.51–2.93) | 2.63× (2.57–2.70) | 0.93× (0.91–0.95) | 2.05× (1.98–2.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,307 | 3.9 | 0.48× (0.47–0.49) | 2.72× (2.63–2.81) | 5.48× (5.36–5.60) | 3.50× (3.33–3.67) | 6.01× (5.88–6.14) | 2.62× (2.43–2.84) | 2.55× (2.49–2.60) | 0.90× (0.88–0.93) | 1.98× (1.92–2.05) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.017 s | 1,146 | 4.2 | 2.97× (2.88–3.07) | 3.60× (3.43–3.79) | 4.29× (4.11–4.48) | 4.82× (4.64–5.01) | 5.77× (5.41–6.16) | 3.23× (3.06–3.41) | 2.71× (2.61–2.81) | 2.23× (2.15–2.32) | 3.48× (3.37–3.60) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.012 s | 1,675 | 5.5 | 2.38× (2.20–2.57) | 3.42× (3.25–3.61) | 4.61× (4.36–4.88) | 4.33× (3.85–4.86) | 6.17× (5.87–6.48) | 4.61× (4.29–4.97) | 3.87× (3.66–4.09) | 3.19× (2.96–3.44) | 4.98× (4.62–5.36) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.014 s | 1,375 | 11.2 | 1.92× (1.78–2.06) | 2.27× (2.11–2.44) | 3.21× (2.91–3.55) | 2.59× (2.30–2.92) | 4.93× (4.60–5.28) | 4.01× (3.67–4.39) | 3.37× (3.16–3.59) | 2.78× (2.56–3.01) | 4.33× (4.02–4.65) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.072 s | 111 | 41.3 | 1.50× (1.48–1.51) | 3.43× (3.35–3.51) | 3.08× (3.02–3.14) | 3.04× (3.00–3.09) | 3.40× (3.32–3.49) | 3.80× (3.72–3.88) | 4.06× (3.70–4.45) | 1.45× (1.42–1.48) | 1.68× (1.65–1.71) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.044 s | 180 | 46.4 | 1.42× (1.34–1.51) | 3.21× (3.08–3.35) | 3.00× (2.86–3.13) | 3.67× (3.45–3.91) | 3.44× (3.29–3.59) | 6.10× (5.83–6.38) | 6.51× (5.86–7.23) | 2.33× (2.22–2.45) | 2.70× (2.58–2.82) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.036 s | 222 | 62.9 | 1.46× (1.40–1.51) | 2.73× (2.62–2.85) | 2.69× (2.64–2.74) | 3.22× (3.11–3.34) | 3.62× (3.41–3.85) | 7.51× (7.26–7.76) | 8.02× (7.28–8.82) | 2.87× (2.81–2.93) | 3.32× (3.22–3.42) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.077 s | 125 | 36.9 | 1.59× (1.58–1.60) | 4.00× (3.93–4.08) | 3.28× (3.23–3.34) | 3.31× (3.25–3.36) | 3.73× (3.69–3.78) | 4.75× (4.67–4.82) | 5.20× (5.10–5.30) | 1.71× (1.68–1.75) | 2.02× (1.91–2.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.047 s | 203 | 42.9 | 1.46× (1.40–1.52) | 3.52× (3.41–3.64) | 3.04× (2.97–3.11) | 4.16× (4.05–4.27) | 3.65× (3.55–3.75) | 7.70× (7.51–7.90) | 8.44× (8.24–8.64) | 2.78× (2.70–2.86) | 3.28× (3.09–3.48) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.040 s | 241 | 57.8 | 1.51× (1.44–1.58) | 2.97× (2.80–3.15) | 2.84× (2.72–2.97) | 3.70× (3.55–3.86) | 3.67× (3.35–4.02) | 9.14× (8.86–9.44) | 10.02× (9.72–10.32) | 3.30× (3.19–3.42) | 3.90× (3.64–4.17) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,105 | 4.6 | 0.63× (0.62–0.64) | 3.39× (3.33–3.44) | 4.84× (4.60–5.09) | 3.23× (3.15–3.30) | 7.36× (7.21–7.52) | 2.37× (2.34–2.41) | 2.46× (2.42–2.50) | 1.01× (0.98–1.05) | 2.26× (2.22–2.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,005 | 5.5 | 0.46× (0.39–0.54) | 2.99× (2.46–3.64) | 4.63× (3.94–5.45) | 2.77× (2.33–3.28) | 6.17× (5.25–7.25) | 2.03× (1.71–2.41) | 2.11× (1.77–2.51) | 0.87× (0.73–1.03) | 1.94× (1.63–2.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 5.5 | 0.48× (0.46–0.50) | 2.97× (2.84–3.10) | 5.12× (4.89–5.35) | 2.89× (2.79–3.00) | 6.45× (6.07–6.85) | 2.07× (1.98–2.16) | 2.15× (2.05–2.24) | 0.88× (0.83–0.94) | 1.97× (1.89–2.06) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.038 s | 396 | 11.8 | 1.88× (1.70–2.08) | 4.37× (3.96–4.83) | 4.67× (4.23–5.16) | 4.92× (4.46–5.43) | 5.14× (4.65–5.68) | 5.06× (4.53–5.65) | 4.93× (4.41–5.52) | 1.62× (1.46–1.79) | 2.06× (1.86–2.27) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.028 s | 538 | 15.7 | 1.44× (1.36–1.52) | 3.40× (3.18–3.63) | 4.15× (3.91–4.40) | 4.50× (4.24–4.78) | 4.51× (4.27–4.77) | 7.05× (6.61–7.51) | 6.88× (6.40–7.39) | 2.26× (2.14–2.39) | 2.86× (2.72–3.02) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.023 s | 638 | 24.0 | 1.56× (1.47–1.65) | 2.67× (2.57–2.77) | 3.77× (3.66–3.88) | 3.87× (3.74–4.00) | 4.44× (4.33–4.56) | 8.45× (8.02–8.90) | 8.25× (7.76–8.77) | 2.71× (2.63–2.79) | 3.44× (3.35–3.53) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.91× (2.86–2.97) | 7.24× (7.15–7.33) | 7.55× (7.48–7.62) | 8.69× (8.59–8.78) | 7.66× (7.59–7.73) | 7.91× (7.82–8.00) | 7.54× (7.45–7.64) | 2.30× (2.25–2.35) | 2.65× (2.60–2.71) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.69–2.80) | 6.90× (6.78–7.02) | 7.52× (7.04–8.03) | 8.51× (8.33–8.70) | 7.19× (7.06–7.33) | 7.36× (7.18–7.54) | 7.01× (6.87–7.16) | 2.14× (2.09–2.18) | 2.47× (2.38–2.55) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.79× (2.70–2.89) | 5.25× (5.19–5.32) | 5.71× (5.60–5.83) | 6.35× (6.28–6.42) | 5.58× (5.54–5.62) | 4.76× (4.72–4.81) | 4.54× (4.48–4.60) | 1.38× (1.36–1.40) | 1.60× (1.57–1.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.61–0.64) | 3.14× (3.04–3.24) | 4.00× (3.89–4.10) | 3.52× (3.45–3.60) | 6.21× (6.02–6.39) | 2.78× (2.53–3.05) | 2.68× (2.61–2.76) | 1.01× (0.98–1.03) | 1.91× (1.86–1.96) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.52–0.54) | 2.80× (2.46–3.18) | 4.66× (4.58–4.73) | 3.16× (3.09–3.23) | 5.37× (5.25–5.50) | 2.45× (2.25–2.68) | 2.37× (2.32–2.42) | 0.89× (0.87–0.91) | 1.68× (1.65–1.72) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.50–0.52) | 2.77× (2.72–2.82) | 4.59× (4.51–4.66) | 3.21× (3.07–3.36) | 5.41× (5.29–5.53) | 2.43× (2.23–2.64) | 2.34× (2.29–2.39) | 0.88× (0.86–0.90) | 1.67× (1.61–1.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.82× (2.69–2.95) | 3.29× (3.13–3.44) | 3.88× (3.70–4.07) | 4.54× (4.31–4.78) | 5.42× (5.01–5.86) | 3.07× (2.87–3.27) | 2.58× (2.45–2.71) | 2.14× (2.03–2.25) | 2.87× (2.75–2.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.38× (2.33–2.42) | 2.86× (2.75–2.97) | 3.32× (3.28–3.37) | 3.74× (3.66–3.82) | 4.28× (4.24–4.32) | 2.41× (2.31–2.51) | 2.03× (1.98–2.07) | 1.68× (1.62–1.75) | 2.26× (2.20–2.31) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.95–2.10) | 1.82× (1.75–1.90) | 2.21× (2.11–2.31) | 2.24× (2.08–2.41) | 2.42× (2.34–2.51) | 1.20× (1.13–1.27) | 1.01× (0.97–1.05) | 0.84× (0.80–0.88) | 1.12× (1.08–1.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.47–1.50) | 3.29× (3.23–3.36) | 2.97× (2.91–3.02) | 3.00× (2.96–3.05) | 3.35× (3.27–3.44) | 3.75× (3.66–3.83) | 4.00× (3.64–4.39) | 1.44× (1.42–1.47) | 1.58× (1.56–1.60) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.39× (1.34–1.44) | 3.03× (2.93–3.12) | 2.61× (2.53–2.70) | 3.44× (3.30–3.58) | 2.97× (2.87–3.08) | 3.29× (3.18–3.41) | 3.51× (3.18–3.88) | 1.27× (1.22–1.32) | 1.39× (1.34–1.44) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.53× (1.46–1.61) | 2.82× (2.75–2.88) | 2.36× (2.30–2.42) | 3.12× (3.04–3.20) | 2.93× (2.82–3.05) | 2.48× (2.41–2.55) | 2.65× (2.40–2.91) | 0.96× (0.94–0.97) | 1.04× (1.02–1.07) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.58× (1.57–1.60) | 3.87× (3.81–3.93) | 3.18× (3.15–3.22) | 3.27× (3.21–3.33) | 3.68× (3.64–3.73) | 4.68× (4.61–4.76) | 5.13× (5.03–5.23) | 1.70× (1.67–1.74) | 1.94× (1.83–2.06) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.45× (1.42–1.48) | 3.47× (3.39–3.55) | 2.76× (2.71–2.81) | 3.87× (3.79–3.95) | 3.20× (3.14–3.26) | 4.05× (3.98–4.12) | 4.43× (4.36–4.51) | 1.47× (1.43–1.51) | 1.68× (1.57–1.79) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.67× (1.62–1.73) | 3.22× (3.12–3.32) | 2.41× (2.35–2.47) | 3.40× (3.31–3.50) | 3.13× (2.99–3.27) | 2.96× (2.87–3.05) | 3.24× (3.14–3.35) | 1.08× (1.04–1.12) | 1.23× (1.14–1.32) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.69× (0.68–0.71) | 2.63× (2.56–2.69) | 3.58× (3.26–3.94) | 2.96× (2.89–3.04) | 6.58× (6.43–6.74) | 2.23× (2.18–2.27) | 2.29× (2.25–2.34) | 1.02× (0.99–1.05) | 1.57× (1.52–1.62) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.51× (0.43–0.61) | 2.13× (1.71–2.64) | 3.55× (2.97–4.25) | 2.43× (2.03–2.92) | 5.18× (4.35–6.16) | 1.78× (1.48–2.14) | 1.83× (1.52–2.21) | 0.81× (0.68–0.98) | 1.26× (1.04–1.52) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.52× (0.49–0.55) | 2.28× (2.15–2.41) | 3.65× (3.42–3.89) | 2.61× (2.49–2.74) | 5.56× (5.18–5.98) | 1.87× (1.76–1.98) | 1.92× (1.82–2.04) | 0.85× (0.80–0.91) | 1.32× (1.25–1.39) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 1.84× (1.66–2.05) | 4.04× (3.67–4.45) | 4.40× (3.98–4.88) | 4.78× (4.31–5.29) | 4.99× (4.49–5.54) | 4.91× (4.37–5.50) | 4.78× (4.26–5.37) | 1.59× (1.43–1.77) | 1.80× (1.65–1.97) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.57× (1.54–1.61) | 3.39× (3.28–3.50) | 3.72× (3.64–3.81) | 4.33× (4.20–4.45) | 4.05× (3.94–4.15) | 3.90× (3.74–4.06) | 3.80× (3.65–3.96) | 1.27× (1.24–1.29) | 1.43× (1.39–1.47) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 1.57× (1.51–1.64) | 2.68× (2.64–2.73) | 2.96× (2.90–3.01) | 3.33× (3.26–3.41) | 3.21× (3.17–3.26) | 2.54× (2.43–2.65) | 2.48× (2.35–2.62) | 0.82× (0.81–0.84) | 0.93× (0.92–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.80× (0.78–0.82) | 11.07× (11.04–11.10) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.35) | 0.53× (0.48–0.59) | 0.70× (0.68–0.72) | 0.60× (0.58–0.61) | 5.66× (5.52–5.79) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 83 MB | 0.38× (0.36–0.39) | 0.49× (0.42–0.57) | 0.67× (0.64–0.70) | 0.58× (0.56–0.61) | 3.88× (3.72–4.05) | 0.09× (0.09–0.10) | 0.10× (0.09–0.10) | 0.15× (0.14–0.16) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.28× (0.27–0.28) | 0.18× (0.17–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.43–0.47) | 1.70× (1.69–1.70) | 0.66× (0.66–0.67) | 8.00× (7.99–8.01) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.65–0.70) | 0.68× (0.63–0.73) | 1.58× (1.53–1.63) | 0.79× (0.77–0.82) | 5.73× (5.56–5.91) | 0.47× (0.45–0.48) | 0.49× (0.47–0.50) | 0.53× (0.52–0.55) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.68× (0.65–0.71) | 0.75× (0.69–0.82) | 1.22× (1.15–1.29) | 0.75× (0.71–0.80) | 3.55× (3.37–3.74) | 0.28× (0.26–0.29) | 0.29× (0.28–0.31) | 0.32× (0.30–0.34) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.18× (5.17–5.19) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.51× (0.48–0.53) | 0.43× (0.41–0.45) | 0.91× (0.87–0.96) | 0.73× (0.69–0.77) | 3.17× (3.03–3.32) | 0.20× (0.19–0.21) | 0.21× (0.20–0.22) | 0.32× (0.30–0.33) | 0.16× (0.15–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.65× (0.63–0.68) | 0.46× (0.44–0.47) | 0.87× (0.85–0.89) | 0.83× (0.80–0.85) | 2.53× (2.46–2.60) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.20–5.24) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.46–0.49) | 0.37× (0.35–0.40) | 0.85× (0.82–0.87) | 0.70× (0.69–0.72) | 2.91× (2.83–3.00) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.60–0.62) | 0.48× (0.45–0.51) | 0.80× (0.79–0.81) | 0.81× (0.78–0.83) | 2.31× (2.27–2.35) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.38× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.35× (5.33–5.37) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.86–1.88) | 0.90× (0.89–0.90) | 6.21× (6.19–6.23) | 0.41× (0.40–0.41) | 0.43× (0.42–0.44) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.89× (0.89–0.90) | 6.19× (6.18–6.21) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.40× (1.39–1.40) | 0.66× (0.66–0.67) | 6.43× (6.40–6.46) | 0.39× (0.38–0.39) | 0.42× (0.41–0.43) | 0.63× (0.63–0.64) | 0.30× (0.30–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 34 MB | 0.49× (0.47–0.51) | 0.43× (0.39–0.48) | 1.11× (1.07–1.14) | 0.72× (0.69–0.75) | 3.61× (3.47–3.75) | 0.22× (0.21–0.23) | 0.24× (0.23–0.25) | 0.36× (0.34–0.37) | 0.17× (0.16–0.18) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 55 MB | 0.51× (0.49–0.53) | 0.44× (0.40–0.48) | 0.92× (0.91–0.94) | 0.76× (0.75–0.78) | 2.35× (2.31–2.38) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 742 | 1.35 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,303 | 1.45 | 1.08× (1.05–1.10) | 88% | 1.03× (1.01–1.04) | 1.01× (1.00–1.03) |
| 4: all 4 CPUs, sharing SMT siblings | 1,575 | 2.26 | 1.66× (1.64–1.68) | 53% | 1.20× (1.18–1.23) | 1.59× (1.51–1.68) |

**Best:** mibz2 at 4 threads (1,575 MB/s); lbzip2 at 4 threads (328 MB/s); pbz2 at 4 threads (520 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.82, mibz2 4t 1.07, lbzip2 1t 0.07, lbzip2 2t 0.34, lbzip2 4t 0.37, pbz2 1t 0.00, pbz2 2t 0.46, pbz2 4t 0.61. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 16%, mibz2 4t 13%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.04 | 0.36 | 0.01 | 0.06 | 0.133 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4528 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 35.5s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.77 | 11.3% |
| MTF + RLE2 | 1.48 | 21.8% |
| T-vector build | 1.30 | 19.3% |
| walk (the one each build uses) | 2.78 | 41.1% |
| of which: segment assembly | 0.11 | 1.7% |
| RLE1 | 0.16 | 2.3% |
| CRC | 0.26 | 3.8% |
| **total** | 6.78 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.287 | 0.694 | (in its total) | 2.43× (2.40–2.46) |  |
| walk | 0.594 | 1.542 | (in its total) | 2.61× (2.56–2.65) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.025 | 0.029 | (in its total) | 1.18× (1.17–1.19) |  |
| inverse BWT, total | 0.906 | 2.264 | 1.257 | 2.51× (2.47–2.55) | 1.39× (1.37–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 41m15s ± 4m58s |
| 4 threads | 19m29s ± 2m48s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m01s ± 1m51s |
| the scanner alone (its serial part) | 5m58s ± 19.7s |
| the workers alone, over 4 | 13m01s ± 1m51s |

At 4 threads the workers bound it. Per byte: the scanner 3.5 ns a byte of bz2, the workers 1.71 ns a byte of output at 4 threads (CPU, summed over them), 1.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,228,219 | 14,890,034 | \`9ad9de226af45720…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4527.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
