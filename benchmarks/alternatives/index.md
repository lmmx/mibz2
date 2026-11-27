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

The latest record: [`40a1616`](<https://github.com/lmmx/mibz2/commit/40a1616520d7fddbc1fb3f156b9cb48fd9af5fee>) (2026-11-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/46080276887>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.9.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.298 s | 702 | 6.0 | 2.97× (2.92–3.02) | 7.52× (7.44–7.60) | 7.81× (7.74–7.87) | 8.91× (8.80–9.02) | 7.98× (7.90–8.05) | 8.14× (8.05–8.23) | 7.76× (7.68–7.85) | 2.36× (2.33–2.39) | 2.66× (2.64–2.68) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.172 s | 1,216 | 6.6 | 2.94× (2.83–3.06) | 6.69× (6.42–6.97) | 8.13× (7.84–8.44) | 8.18× (7.92–8.45) | 7.32× (7.04–7.61) | 14.02× (13.53–14.54) | 13.38× (12.94–13.83) | 4.06× (3.91–4.22) | 4.59× (4.42–4.76) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.142 s | 1,482 | 9.9 | 2.99× (2.94–3.04) | 4.83× (4.77–4.89) | 6.35× (6.21–6.49) | 5.86× (5.75–5.97) | 5.91× (5.79–6.02) | 17.03× (16.65–17.41) | 16.24× (15.89–16.60) | 4.93× (4.80–5.06) | 5.57× (5.41–5.73) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,351 | 3.5 | 0.57× (0.56–0.58) | 3.69× (3.29–4.14) | 4.43× (4.22–4.66) | 3.73× (3.66–3.79) | 6.55× (6.45–6.66) | 2.85× (2.74–2.96) | 2.79× (2.74–2.85) | 1.02× (0.94–1.10) | 2.07× (2.03–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 s | 1,262 | 3.9 | 0.49× (0.48–0.50) | 3.10× (2.89–3.32) | 5.47× (5.25–5.70) | 3.48× (3.40–3.57) | 6.02× (5.89–6.15) | 2.67× (2.58–2.76) | 2.61× (2.56–2.67) | 0.96× (0.89–1.02) | 1.94× (1.90–1.98) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,258 | 3.8 | 0.48× (0.47–0.50) | 3.10× (2.93–3.29) | 5.34× (5.18–5.52) | 3.53× (3.26–3.81) | 5.94× (5.74–6.16) | 2.61× (2.49–2.73) | 2.56× (2.48–2.64) | 0.93× (0.85–1.02) | 1.90× (1.83–1.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,043 | 4.4 | 2.87× (2.67–3.10) | 3.69× (3.41–3.99) | 4.29× (3.96–4.64) | 4.60× (4.26–4.98) | 5.52× (5.09–5.97) | 3.10× (2.81–3.41) | 2.57× (2.39–2.76) | 2.16× (2.01–2.33) | 3.16× (2.92–3.41) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,457 | 5.6 | 2.11× (1.77–2.51) | 2.88× (2.45–3.39) | 3.90× (3.23–4.70) | 3.65× (3.10–4.30) | 5.41× (4.56–6.41) | 4.03× (3.39–4.79) | 3.34× (2.81–3.97) | 2.81× (2.37–3.33) | 4.10× (3.46–4.87) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.015 s | 1,320 | 10.8 | 1.90× (1.79–2.02) | 2.15× (2.03–2.28) | 3.16× (2.87–3.49) | 2.47× (2.30–2.65) | 4.92× (4.66–5.19) | 4.05× (3.77–4.35) | 3.36× (3.16–3.56) | 2.82× (2.66–2.99) | 4.12× (3.89–4.37) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.077 s | 104 | 41.2 | 1.50× (1.49–1.51) | 3.50× (3.43–3.58) | 3.16× (3.05–3.28) | 3.08× (3.03–3.13) | 3.42× (3.39–3.46) | 3.86× (3.79–3.94) | 3.92× (3.85–4.00) | 1.47× (1.46–1.48) | 1.70× (1.67–1.73) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.047 s | 169 | 47.3 | 1.39× (1.32–1.46) | 3.09× (2.94–3.26) | 2.90× (2.80–3.00) | 3.65× (3.46–3.84) | 3.51× (3.31–3.74) | 6.13× (5.90–6.36) | 6.22× (5.97–6.48) | 2.33× (2.24–2.41) | 2.69× (2.58–2.81) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.039 s | 205 | 62.0 | 1.44× (1.41–1.48) | 2.74× (2.64–2.84) | 2.70× (2.63–2.78) | 3.19× (3.09–3.31) | 3.60× (3.46–3.75) | 7.59× (7.33–7.85) | 7.71× (7.48–7.93) | 2.88× (2.82–2.94) | 3.34× (3.22–3.46) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.082 s | 117 | 36.7 | 1.58× (1.57–1.60) | 4.07× (3.98–4.17) | 3.38× (3.32–3.43) | 3.38× (3.33–3.42) | 3.82× (3.77–3.87) | 4.79× (4.72–4.87) | 5.15× (5.09–5.21) | 1.79× (1.72–1.86) | 1.98× (1.95–2.02) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.052 s | 183 | 42.4 | 1.41× (1.35–1.46) | 3.55× (3.37–3.74) | 3.08× (2.97–3.19) | 4.16× (4.01–4.32) | 3.64× (3.54–3.74) | 7.55× (7.34–7.76) | 8.11× (7.88–8.35) | 2.82× (2.70–2.94) | 3.12× (3.05–3.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.044 s | 219 | 59.8 | 1.40× (1.21–1.63) | 2.67× (2.38–3.00) | 2.52× (2.23–2.85) | 3.42× (3.01–3.89) | 3.18× (2.82–3.57) | 8.29× (7.41–9.28) | 8.91× (7.97–9.97) | 3.10× (2.72–3.53) | 3.43× (3.07–3.84) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,015 | 4.8 | 0.61× (0.59–0.63) | 3.09× (3.01–3.17) | 4.27× (4.16–4.39) | 3.14× (3.06–3.21) | 7.02× (6.85–7.19) | 2.33× (2.26–2.39) | 2.35× (2.28–2.41) | 1.03× (0.91–1.18) | 2.07× (2.01–2.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 s | 952 | 5.4 | 0.49× (0.47–0.51) | 2.88× (2.78–2.99) | 4.96× (4.79–5.13) | 2.93× (2.83–3.03) | 6.55× (6.32–6.79) | 2.17× (2.09–2.24) | 2.19× (2.11–2.26) | 0.96× (0.87–1.06) | 1.93× (1.86–1.99) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 913 | 5.4 | 0.54× (0.47–0.61) | 2.82× (2.76–2.88) | 4.83× (4.73–4.94) | 2.86× (2.81–2.91) | 6.41× (6.25–6.58) | 2.11× (2.08–2.16) | 2.13× (2.09–2.18) | 0.94× (0.83–1.06) | 1.88× (1.84–1.92) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 0.041 s | 364 | 12.0 | 1.95× (1.90–2.00) | 4.53× (4.37–4.69) | 4.89× (4.76–5.02) | 5.10× (5.00–5.20) | 5.43× (5.27–5.59) | 5.12× (5.02–5.22) | 4.95× (4.84–5.06) | 1.67× (1.64–1.71) | 2.00× (1.95–2.06) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 0.030 s | 503 | 15.4 | 1.44× (1.34–1.55) | 3.41× (3.20–3.64) | 4.06× (3.73–4.42) | 4.58× (4.14–5.06) | 4.46× (4.09–4.87) | 6.84× (6.29–7.44) | 6.61× (6.07–7.20) | 2.24× (2.05–2.44) | 2.67× (2.45–2.92) | yes |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 0.027 s | 545 | 23.8 | 1.39× (1.33–1.45) | 2.42× (2.23–2.61) | 3.45× (3.28–3.63) | 3.54× (3.37–3.71) | 4.07× (3.89–4.26) | 7.62× (7.29–7.96) | 7.36× (7.04–7.69) | 2.49× (2.38–2.61) | 2.98× (2.80–3.16) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.96× (2.91–3.01) | 7.47× (7.39–7.55) | 7.76× (7.69–7.82) | 8.87× (8.76–8.98) | 7.93× (7.87–8.00) | 8.10× (8.01–8.19) | 7.73× (7.64–7.82) | 2.35× (2.32–2.38) | 2.64× (2.62–2.66) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.77× (2.74–2.80) | 6.96× (6.80–7.12) | 7.39× (7.24–7.54) | 8.48× (8.35–8.62) | 7.25× (7.12–7.38) | 7.41× (7.29–7.53) | 7.07× (7.00–7.15) | 2.15× (2.12–2.18) | 2.42× (2.38–2.45) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.86× (2.81–2.90) | 5.27× (5.15–5.40) | 5.82× (5.66–5.99) | 6.51× (6.41–6.61) | 5.70× (5.56–5.85) | 4.84× (4.70–4.99) | 4.62× (4.48–4.77) | 1.41× (1.36–1.45) | 1.58× (1.53–1.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.62× (0.61–0.63) | 3.27× (2.91–3.68) | 3.92× (3.85–3.99) | 3.47× (3.41–3.52) | 6.03× (5.94–6.12) | 2.68× (2.58–2.77) | 2.62× (2.57–2.67) | 1.02× (0.95–1.09) | 1.85× (1.81–1.89) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.53× (0.52–0.54) | 2.64× (2.59–2.69) | 4.65× (4.57–4.73) | 3.15× (3.09–3.21) | 5.35× (5.26–5.45) | 2.42× (2.34–2.50) | 2.36× (2.32–2.41) | 0.92× (0.85–0.99) | 1.67× (1.64–1.70) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.51× (0.50–0.53) | 2.82× (2.72–2.92) | 4.63× (4.48–4.78) | 3.25× (3.00–3.52) | 5.35× (5.16–5.55) | 2.41× (2.29–2.53) | 2.35× (2.28–2.43) | 0.91× (0.84–1.00) | 1.66× (1.61–1.72) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.74× (2.55–2.94) | 3.18× (2.96–3.42) | 3.78× (3.52–4.07) | 4.34× (4.03–4.67) | 5.18× (4.81–5.59) | 2.95× (2.69–3.23) | 2.45× (2.30–2.62) | 2.07× (1.93–2.22) | 2.79× (2.60–3.00) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.36× (2.32–2.41) | 2.78× (2.73–2.84) | 3.29× (3.21–3.37) | 3.69× (3.60–3.78) | 4.25× (4.15–4.36) | 2.36× (2.25–2.48) | 1.96× (1.92–2.00) | 1.66× (1.63–1.69) | 2.24× (2.18–2.30) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.00× (1.89–2.12) | 1.88× (1.74–2.02) | 2.19× (2.06–2.32) | 2.25× (2.12–2.38) | 2.47× (2.35–2.60) | 1.21× (1.13–1.28) | 1.00× (0.95–1.06) | 0.85× (0.80–0.90) | 1.14× (1.08–1.21) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.49× (1.48–1.50) | 3.36× (3.29–3.43) | 3.06× (2.94–3.19) | 3.04× (2.99–3.09) | 3.38× (3.34–3.41) | 3.81× (3.74–3.88) | 3.87× (3.80–3.94) | 1.46× (1.45–1.47) | 1.59× (1.58–1.61) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.36× (1.35–1.38) | 3.00× (2.97–3.04) | 2.61× (2.57–2.65) | 3.39× (3.34–3.44) | 3.06× (2.89–3.24) | 3.31× (3.24–3.38) | 3.36× (3.29–3.43) | 1.27× (1.25–1.29) | 1.39× (1.36–1.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.57× (1.53–1.61) | 2.89× (2.80–2.98) | 2.34× (2.29–2.39) | 3.11× (3.03–3.20) | 2.98× (2.91–3.05) | 2.53× (2.44–2.62) | 2.57× (2.51–2.63) | 0.97× (0.95–0.99) | 1.06× (1.03–1.08) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.57× (1.56–1.59) | 3.97× (3.88–4.06) | 3.27× (3.22–3.31) | 3.33× (3.29–3.38) | 3.77× (3.72–3.82) | 4.72× (4.65–4.80) | 5.07× (5.01–5.14) | 1.78× (1.71–1.84) | 1.91× (1.89–1.93) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.44× (1.42–1.45) | 3.55× (3.46–3.64) | 2.82× (2.74–2.90) | 3.93× (3.85–4.01) | 3.28× (3.21–3.36) | 4.06× (3.97–4.15) | 4.36× (4.28–4.44) | 1.53× (1.47–1.59) | 1.64× (1.61–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.57× (1.32–1.86) | 2.93× (2.56–3.34) | 2.22× (1.96–2.51) | 3.16× (2.78–3.58) | 2.84× (2.50–3.21) | 2.73× (2.42–3.09) | 2.94× (2.60–3.32) | 1.03× (0.90–1.18) | 1.11× (0.97–1.25) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.54× (2.50–2.59) | 3.33× (3.27–3.39) | 2.86× (2.83–2.90) | 6.24× (6.15–6.32) | 2.16× (2.12–2.20) | 2.17× (2.14–2.21) | 1.04× (0.89–1.21) | 1.55× (1.42–1.68) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.53–0.57) | 2.18× (2.10–2.26) | 3.72× (3.61–3.83) | 2.59× (2.51–2.67) | 5.53× (5.36–5.72) | 1.90× (1.85–1.96) | 1.91× (1.86–1.97) | 0.92× (0.81–1.04) | 1.36× (1.24–1.50) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.50–0.66) | 2.32× (2.28–2.37) | 3.70× (3.63–3.77) | 2.60× (2.56–2.64) | 5.58× (5.44–5.72) | 1.92× (1.89–1.95) | 1.93× (1.90–1.96) | 0.92× (0.80–1.06) | 1.37× (1.27–1.49) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 1.92× (1.86–1.97) | 4.28× (4.13–4.43) | 4.64× (4.53–4.75) | 4.95× (4.85–5.05) | 5.27× (5.11–5.43) | 4.97× (4.87–5.07) | 4.80× (4.69–4.91) | 1.65× (1.62–1.68) | 1.86× (1.80–1.92) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 1.56× (1.52–1.60) | 3.42× (3.28–3.56) | 3.74× (3.65–3.84) | 4.36× (4.21–4.52) | 4.08× (3.99–4.17) | 3.86× (3.77–3.95) | 3.73× (3.63–3.83) | 1.28× (1.25–1.31) | 1.44× (1.39–1.50) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 1.57× (1.50–1.64) | 2.71× (2.63–2.78) | 2.98× (2.88–3.08) | 3.35× (3.28–3.42) | 3.22× (3.14–3.31) | 2.50× (2.44–2.55) | 2.41× (2.35–2.47) | 0.83× (0.81–0.85) | 0.93× (0.90–0.97) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.34× (0.33–0.35) | 0.98× (0.98–0.98) | 0.78× (0.76–0.79) | 11.06× (11.04–11.07) | 0.27× (0.26–0.27) | 0.28× (0.28–0.29) | 0.43× (0.43–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.49× (0.41–0.59) | 0.70× (0.68–0.72) | 0.59× (0.57–0.61) | 5.59× (5.44–5.75) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 86 MB | 0.35× (0.34–0.37) | 0.55× (0.43–0.70) | 0.63× (0.60–0.66) | 0.56× (0.53–0.58) | 3.66× (3.49–3.83) | 0.09× (0.08–0.09) | 0.09× (0.09–0.10) | 0.14× (0.13–0.15) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.28) | 0.18× (0.18–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.95× (1.95–1.96) | 0.93× (0.92–0.93) | 4.84× (4.83–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.22) | 0.18× (0.18–0.18) | 1.96× (1.96–1.96) | 0.93× (0.92–0.93) | 4.86× (4.85–4.87) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.03–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.44–0.46) | 1.70× (1.70–1.71) | 0.66× (0.66–0.67) | 8.04× (8.01–8.07) | 0.66× (0.65–0.66) | 0.69× (0.69–0.69) | 0.75× (0.75–0.76) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.66–0.71) | 0.66× (0.59–0.73) | 1.62× (1.58–1.65) | 0.81× (0.80–0.83) | 5.87× (5.75–5.99) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.55× (0.53–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.67× (0.64–0.70) | 0.76× (0.73–0.80) | 1.22× (1.17–1.27) | 0.75× (0.73–0.78) | 3.56× (3.42–3.70) | 0.28× (0.27–0.29) | 0.29× (0.28–0.31) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.18× (5.17–5.19) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.49× (0.48–0.51) | 0.42× (0.41–0.43) | 0.89× (0.87–0.91) | 0.72× (0.70–0.74) | 3.09× (3.01–3.17) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.66) | 0.46× (0.45–0.47) | 0.87× (0.85–0.89) | 0.82× (0.80–0.84) | 2.54× (2.49–2.58) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.84) | 5.22× (5.21–5.24) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.49) | 0.38× (0.36–0.40) | 0.84× (0.82–0.88) | 0.71× (0.67–0.75) | 2.90× (2.80–3.01) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.47× (0.43–0.50) | 0.80× (0.78–0.82) | 0.83× (0.80–0.86) | 2.31× (2.27–2.36) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.22× (0.21–0.23) | 1.07× (1.06–1.07) | 0.76× (0.76–0.77) | 5.35× (5.33–5.36) | 0.35× (0.35–0.35) | 0.37× (0.36–0.38) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.28× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.87–1.87) | 0.90× (0.90–0.90) | 6.20× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.24× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.90) | 6.20× (6.19–6.22) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.06–1.06) | 0.15× (0.15–0.15) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 1 | 20 MB | 0.46× (0.46–0.47) | 0.38× (0.38–0.39) | 1.40× (1.40–1.41) | 0.66× (0.66–0.67) | 6.45× (6.43–6.48) | 0.39× (0.39–0.39) | 0.42× (0.41–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 2 | 33 MB | 0.51× (0.50–0.53) | 0.43× (0.39–0.48) | 1.16× (1.12–1.20) | 0.78× (0.75–0.81) | 3.80× (3.68–3.92) | 0.23× (0.22–0.24) | 0.25× (0.24–0.26) | 0.37× (0.36–0.39) | 0.18× (0.18–0.19) |
| `text` (2.23 MB → 14.9 MB, 6.7×) | 4 | 53 MB | 0.53× (0.51–0.54) | 0.51× (0.48–0.55) | 0.96× (0.94–0.97) | 0.80× (0.78–0.81) | 2.47× (2.43–2.51) | 0.14× (0.14–0.15) | 0.15× (0.15–0.16) | 0.23× (0.23–0.24) | 0.11× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 702 | 1.43 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,216 | 1.56 | 1.09× (1.08–1.11) | 86% | 1.02× (1.00–1.03) | 1.02× (1.00–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,482 | 2.36 | 1.67× (1.62–1.72) | 52% | 1.18× (1.16–1.20) | 1.62× (1.55–1.69) |

**Best:** mibz2 at 4 threads (1,482 MB/s); lbzip2 at 4 threads (306 MB/s); pbz2 at 4 threads (496 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.85, mibz2 4t 1.06, lbzip2 1t 0.07, lbzip2 2t 0.28, lbzip2 4t 0.47, pbz2 1t 0.00, pbz2 2t 0.44, pbz2 4t 0.58. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 15%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.04 | 0.37 | 0.01 | 0.07 | 0.142 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4216 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.76 | 11.1% |
| MTF + RLE2 | 1.48 | 21.8% |
| T-vector build | 1.30 | 19.1% |
| walk (the one each build uses) | 2.84 | 41.8% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.3% |
| CRC | 0.25 | 3.7% |
| **total** | 6.80 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.307 | 0.737 | (in its total) | 2.41× (2.38–2.44) |  |
| walk | 0.663 | 1.665 | (in its total) | 2.53× (2.48–2.57) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.031 | (in its total) | 1.14× (1.10–1.18) |  |
| inverse BWT, total | 0.997 | 2.432 | 1.351 | 2.45× (2.42–2.49) | 1.37× (1.35–1.39) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 43m33s ± 5m19s |
| 4 threads | 20m50s ± 2m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 13m45s ± 2m01s |
| the scanner alone (its serial part) | 5m49s ± 0.00s |
| the workers alone, over 4 | 13m45s ± 2m01s |

At 4 threads the workers bound it. Per byte: the scanner 3.4 ns a byte of bz2, the workers 1.80 ns a byte of output at 4 threads (CPU, summed over them), 1.24 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,227,455 | 14,888,808 | \`c0a0f7a4c597a907…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4215.8 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261122.9.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.9.0 (bench/report.py), 10 rounds, threads 1, 2, 4
