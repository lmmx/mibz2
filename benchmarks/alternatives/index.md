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

The latest record: [`fb22574`](<https://github.com/lmmx/mibz2/commit/fb2257497ab9fe26ae833b279b6086538a216dfd>) (2026-11-15).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43725845524>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.325 s | 646 | 4.0 | 2.65× (2.59–2.71) | 6.62× (6.44–6.80) | 6.85× (6.69–7.02) | 7.83× (7.63–8.05) | 6.97× (6.79–7.14) | 7.21× (7.02–7.41) | 6.82× (6.64–7.00) | 2.04× (1.99–2.10) | 2.38× (2.32–2.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.195 s | 1,074 | 4.4 | 2.65× (2.52–2.78) | 5.86× (5.72–6.00) | 6.94× (6.71–7.17) | 7.02× (6.83–7.21) | 6.28× (6.08–6.48) | 12.12× (11.77–12.48) | 11.45× (11.07–11.85) | 3.44× (3.33–3.55) | 4.00× (3.86–4.14) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.160 s | 1,308 | 6.9 | 2.60× (2.50–2.71) | 4.11× (3.86–4.37) | 5.33× (5.04–5.64) | 4.93× (4.62–5.27) | 4.93× (4.56–5.32) | 14.16× (13.00–15.43) | 13.38× (12.27–14.59) | 4.01× (3.66–4.41) | 4.67× (4.30–5.07) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,408 | 2.0 | 0.57× (0.55–0.58) | 3.53× (3.41–3.65) | 4.35× (4.19–4.50) | 3.70× (3.58–3.82) | 6.54× (6.29–6.79) | 2.83× (2.72–2.93) | 2.80× (2.70–2.89) | 1.00× (0.96–1.04) | 2.12× (2.05–2.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.014 s | 1,389 | 2.1 | 0.52× (0.51–0.53) | 2.89× (2.78–3.01) | 5.79× (5.63–5.95) | 3.74× (3.64–3.84) | 6.42× (6.26–6.59) | 2.82× (2.74–2.91) | 2.79× (2.72–2.87) | 1.00× (0.98–1.02) | 2.12× (2.06–2.17) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,324 | 2.2 | 0.51× (0.50–0.52) | 2.86× (2.68–3.06) | 5.57× (5.45–5.70) | 3.62× (3.53–3.71) | 6.28× (6.11–6.46) | 2.73× (2.66–2.79) | 2.70× (2.64–2.75) | 0.96× (0.92–1.00) | 2.04× (2.00–2.09) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.019 s | 1,054 | 2.7 | 3.04× (2.70–3.42) | 3.84× (3.76–3.93) | 4.40× (4.30–4.50) | 4.66× (4.56–4.76) | 5.40× (5.27–5.54) | 3.12× (3.01–3.23) | 2.70× (2.51–2.91) | 2.17× (2.12–2.22) | 3.28× (3.21–3.36) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,457 | 3.5 | 2.24× (2.08–2.42) | 3.04× (2.83–3.26) | 4.37× (4.04–4.74) | 3.99× (3.73–4.28) | 5.76× (5.37–6.18) | 4.28× (3.96–4.63) | 3.71× (3.36–4.09) | 2.98× (2.79–3.19) | 4.51× (4.21–4.84) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,160 | 7.1 | 1.65× (1.57–1.73) | 1.89× (1.79–2.00) | 2.66× (2.56–2.77) | 2.12× (1.99–2.25) | 4.23× (4.07–4.39) | 3.44× (3.28–3.60) | 2.98× (2.76–3.21) | 2.39× (2.29–2.50) | 3.62× (3.47–3.77) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 96 | 27.6 | 1.34× (1.32–1.36) | 3.06× (2.97–3.16) | 2.74× (2.68–2.79) | 2.70× (2.62–2.79) | 3.13× (2.98–3.28) | 3.39× (3.36–3.42) | 3.43× (3.41–3.46) | 1.30× (1.29–1.31) | 1.48× (1.44–1.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.053 s | 152 | 31.7 | 1.20× (1.13–1.27) | 2.83× (2.65–3.03) | 2.55× (2.39–2.72) | 3.20× (3.04–3.37) | 2.98× (2.83–3.14) | 5.25× (4.98–5.53) | 5.31× (5.03–5.62) | 2.01× (1.90–2.12) | 2.29× (2.16–2.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.044 s | 183 | 43.2 | 1.22× (1.15–1.30) | 2.38× (2.21–2.56) | 2.35× (2.22–2.49) | 2.62× (2.48–2.76) | 2.99× (2.83–3.16) | 6.34× (5.99–6.70) | 6.41× (6.08–6.76) | 2.42× (2.29–2.56) | 2.76× (2.62–2.92) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 24.0 | 1.51× (1.43–1.61) | 3.71× (3.62–3.80) | 3.02× (2.95–3.09) | 3.00× (2.93–3.07) | 3.45× (3.40–3.51) | 4.40× (4.28–4.53) | 4.67× (4.51–4.83) | 1.57× (1.54–1.60) | 1.84× (1.78–1.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.056 s | 171 | 27.8 | 1.37× (1.24–1.52) | 3.19× (3.08–3.30) | 2.80× (2.66–2.94) | 3.73× (3.42–4.06) | 3.33× (3.24–3.43) | 6.97× (6.78–7.17) | 7.40× (7.13–7.67) | 2.48× (2.43–2.54) | 2.91× (2.82–3.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.047 s | 205 | 39.1 | 1.37× (1.31–1.43) | 2.66× (2.54–2.79) | 2.51× (2.46–2.56) | 3.25× (3.16–3.33) | 3.09× (3.04–3.15) | 8.24× (8.04–8.45) | 8.74× (8.55–8.94) | 2.94× (2.91–2.97) | 3.44× (3.37–3.50) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,047 | 2.9 | 0.62× (0.61–0.64) | 3.19× (3.12–3.27) | 4.43× (4.33–4.54) | 3.15× (3.08–3.22) | 7.09× (6.95–7.24) | 2.39× (2.29–2.50) | 2.36× (2.30–2.43) | 0.97× (0.95–0.99) | 2.15× (2.08–2.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,000 | 3.1 | 0.51× (0.50–0.52) | 3.08× (3.03–3.13) | 5.27× (5.19–5.34) | 3.05× (3.01–3.09) | 6.78× (6.66–6.91) | 2.29× (2.21–2.37) | 2.26× (2.23–2.30) | 0.93× (0.92–0.95) | 2.06× (2.01–2.12) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 976 | 3.0 | 0.50× (0.49–0.51) | 3.00× (2.95–3.05) | 5.15× (5.06–5.24) | 3.00× (2.94–3.06) | 6.65× (6.42–6.89) | 2.24× (2.15–2.32) | 2.21× (2.16–2.27) | 0.91× (0.89–0.93) | 2.02× (1.97–2.07) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 0.043 s | 294 | 9.1 | 1.72× (1.69–1.75) | 4.25× (4.18–4.33) | 4.53× (4.44–4.62) | 4.75× (4.64–4.86) | 5.02× (4.93–5.11) | 5.05× (4.96–5.15) | 4.85× (4.77–4.94) | 1.51× (1.49–1.54) | 1.81× (1.66–1.97) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 0.031 s | 406 | 11.9 | 1.42× (1.27–1.60) | 3.22× (3.06–3.39) | 3.76× (3.64–3.89) | 4.22× (4.01–4.45) | 4.30× (4.09–4.52) | 6.87× (6.51–7.24) | 6.59× (6.24–6.97) | 2.06× (1.96–2.16) | 2.46× (2.24–2.70) | yes |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 0.029 s | 439 | 17.9 | 1.24× (1.22–1.27) | 2.27× (2.15–2.40) | 3.08× (2.94–3.23) | 3.33× (3.26–3.39) | 3.82× (3.76–3.89) | 7.62× (7.45–7.79) | 7.31× (7.16–7.47) | 2.28× (2.24–2.33) | 2.73× (2.50–2.97) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.64× (2.58–2.70) | 6.58× (6.40–6.76) | 6.81× (6.65–6.97) | 7.80× (7.59–8.01) | 6.93× (6.76–7.11) | 7.18× (6.99–7.38) | 6.78× (6.61–6.96) | 2.04× (1.98–2.10) | 2.36× (2.30–2.41) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.46× (2.40–2.51) | 6.15× (6.05–6.25) | 6.49× (6.39–6.60) | 7.48× (7.31–7.65) | 6.36× (6.22–6.50) | 6.55× (6.41–6.70) | 6.19× (6.07–6.32) | 1.86× (1.82–1.90) | 2.15× (2.10–2.20) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.54× (2.46–2.63) | 4.55× (4.34–4.78) | 5.10× (4.94–5.27) | 5.58× (5.28–5.89) | 4.84× (4.54–5.17) | 4.11× (3.80–4.44) | 3.88× (3.60–4.19) | 1.17× (1.07–1.27) | 1.35× (1.25–1.45) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.59–0.64) | 3.07× (2.92–3.22) | 3.87× (3.67–4.07) | 3.43× (3.27–3.59) | 6.00× (5.68–6.34) | 2.65× (2.51–2.79) | 2.61× (2.49–2.74) | 0.99× (0.94–1.05) | 1.83× (1.75–1.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.58× (0.57–0.60) | 2.88× (2.83–2.94) | 5.03× (4.93–5.14) | 3.44× (3.35–3.53) | 5.81× (5.66–5.96) | 2.60× (2.53–2.68) | 2.57× (2.50–2.64) | 0.98× (0.96–1.00) | 1.80× (1.76–1.85) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.56× (0.54–0.57) | 3.03× (2.96–3.09) | 5.00× (4.89–5.11) | 3.41× (3.32–3.50) | 5.80× (5.65–5.95) | 2.58× (2.52–2.64) | 2.54× (2.50–2.59) | 0.97× (0.93–1.00) | 1.79× (1.74–1.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.87× (2.56–3.22) | 3.20× (3.13–3.27) | 3.80× (3.72–3.89) | 4.35× (4.26–4.44) | 5.04× (4.91–5.17) | 2.94× (2.84–3.04) | 2.55× (2.37–2.75) | 2.07× (2.03–2.11) | 2.79× (2.74–2.83) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.25× (2.15–2.36) | 2.64× (2.53–2.76) | 3.18× (3.04–3.31) | 3.67× (3.33–4.04) | 4.01× (3.85–4.19) | 2.25× (2.12–2.40) | 1.96× (1.81–2.11) | 1.58× (1.52–1.66) | 2.13× (2.04–2.23) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.78–1.93) | 1.70× (1.64–1.75) | 1.98× (1.91–2.04) | 2.05× (1.98–2.13) | 2.22× (2.14–2.30) | 1.09× (1.03–1.15) | 0.94× (0.87–1.02) | 0.76× (0.74–0.79) | 1.03× (0.99–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.32–1.35) | 2.97× (2.89–3.06) | 2.67× (2.63–2.70) | 2.67× (2.59–2.76) | 3.09× (2.95–3.24) | 3.34× (3.32–3.37) | 3.38× (3.36–3.41) | 1.29× (1.28–1.30) | 1.43× (1.42–1.44) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.20× (1.14–1.27) | 2.61× (2.49–2.75) | 2.26× (2.14–2.38) | 2.92× (2.83–3.03) | 2.59× (2.45–2.74) | 2.85× (2.70–3.00) | 2.88× (2.73–3.04) | 1.10× (1.04–1.16) | 1.22× (1.15–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.34× (1.30–1.38) | 2.49× (2.41–2.57) | 2.04× (1.99–2.08) | 2.61× (2.54–2.67) | 2.51× (2.45–2.57) | 2.14× (2.09–2.19) | 2.16× (2.12–2.21) | 0.83× (0.81–0.84) | 0.91× (0.89–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.51× (1.42–1.60) | 3.61× (3.50–3.72) | 2.92× (2.86–2.97) | 2.97× (2.90–3.03) | 3.41× (3.36–3.47) | 4.34× (4.22–4.47) | 4.60× (4.45–4.76) | 1.56× (1.53–1.59) | 1.74× (1.70–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.31× (1.30–1.33) | 3.17× (3.10–3.25) | 2.57× (2.47–2.67) | 3.40× (3.33–3.47) | 2.97× (2.93–3.02) | 3.74× (3.64–3.84) | 3.97× (3.85–4.09) | 1.34× (1.32–1.36) | 1.50× (1.48–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.51× (1.47–1.56) | 2.90× (2.79–3.01) | 2.16× (2.09–2.24) | 2.96× (2.88–3.05) | 2.75× (2.70–2.80) | 2.69× (2.61–2.76) | 2.85× (2.76–2.94) | 0.96× (0.95–0.98) | 1.07× (1.05–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.68× (0.66–0.69) | 2.55× (2.49–2.62) | 3.35× (3.28–3.42) | 2.84× (2.77–2.92) | 6.24× (6.08–6.41) | 2.22× (2.10–2.34) | 2.17× (2.11–2.24) | 0.98× (0.96–1.00) | 1.49× (1.45–1.54) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.59× (0.58–0.60) | 2.35× (2.32–2.38) | 3.99× (3.90–4.08) | 2.77× (2.73–2.81) | 5.89× (5.78–6.02) | 2.08× (1.99–2.18) | 2.04× (2.01–2.07) | 0.92× (0.90–0.94) | 1.41× (1.38–1.43) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.49× (2.45–2.53) | 3.96× (3.88–4.05) | 2.82× (2.76–2.88) | 5.97× (5.77–6.18) | 2.11× (2.01–2.21) | 2.07× (2.03–2.11) | 0.93× (0.91–0.95) | 1.42× (1.39–1.45) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 1.71× (1.68–1.73) | 4.04× (3.98–4.10) | 4.36× (4.31–4.41) | 4.66× (4.57–4.75) | 4.92× (4.85–4.98) | 4.95× (4.87–5.03) | 4.75× (4.69–4.82) | 1.51× (1.49–1.53) | 1.72× (1.58–1.87) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 1.41× (1.38–1.43) | 3.22× (3.17–3.26) | 3.53× (3.47–3.60) | 4.04× (3.96–4.12) | 3.84× (3.75–3.92) | 3.79× (3.70–3.87) | 3.63× (3.55–3.72) | 1.15× (1.14–1.17) | 1.31× (1.20–1.43) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 1.50× (1.46–1.53) | 2.61× (2.53–2.69) | 2.85× (2.76–2.95) | 3.20× (3.14–3.27) | 3.16× (3.08–3.25) | 2.50× (2.43–2.57) | 2.40× (2.33–2.47) | 0.76× (0.75–0.78) | 0.87× (0.80–0.94) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.31) | 0.34× (0.32–0.35) | 0.98× (0.97–0.98) | 0.79× (0.78–0.81) | 11.05× (11.02–11.07) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.43× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 61 MB | 0.32× (0.30–0.33) | 0.49× (0.41–0.58) | 0.65× (0.62–0.67) | 0.56× (0.53–0.58) | 5.19× (4.98–5.41) | 0.12× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.19–0.21) | 0.10× (0.10–0.10) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 92 MB | 0.32× (0.31–0.33) | 0.42× (0.38–0.46) | 0.57× (0.55–0.60) | 0.49× (0.47–0.52) | 3.32× (3.18–3.46) | 0.08× (0.07–0.08) | 0.08× (0.08–0.09) | 0.13× (0.12–0.13) | 0.06× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.18–0.18) | 0.81× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.31) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.83–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.52–0.52) | 0.45× (0.44–0.46) | 1.71× (1.70–1.72) | 0.66× (0.65–0.66) | 8.07× (8.03–8.11) | 0.65× (0.65–0.66) | 0.69× (0.69–0.70) | 0.75× (0.74–0.76) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 23 MB | 0.66× (0.63–0.70) | 0.59× (0.53–0.66) | 1.57× (1.50–1.65) | 0.79× (0.76–0.83) | 5.72× (5.47–5.99) | 0.46× (0.44–0.48) | 0.49× (0.47–0.51) | 0.53× (0.51–0.56) | 0.26× (0.25–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.64× (0.61–0.67) | 0.72× (0.69–0.75) | 1.15× (1.11–1.19) | 0.72× (0.69–0.75) | 3.36× (3.26–3.47) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.21× (5.20–5.22) | 0.33× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.53–0.54) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.48× (0.46–0.51) | 0.40× (0.38–0.42) | 0.87× (0.83–0.91) | 0.70× (0.67–0.73) | 3.02× (2.89–3.15) | 0.19× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 49 MB | 0.64× (0.62–0.67) | 0.46× (0.44–0.47) | 0.87× (0.85–0.90) | 0.82× (0.79–0.85) | 2.53× (2.46–2.60) | 0.15× (0.15–0.15) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.32× (0.31–0.32) | 1.18× (1.18–1.18) | 0.83× (0.83–0.83) | 5.22× (5.21–5.23) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.37× (0.33–0.40) | 0.85× (0.82–0.88) | 0.73× (0.69–0.77) | 2.93× (2.83–3.03) | 0.18× (0.17–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 55 MB | 0.60× (0.58–0.61) | 0.47× (0.44–0.50) | 0.79× (0.78–0.80) | 0.78× (0.75–0.82) | 2.28× (2.24–2.32) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.20–0.23) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.38× (5.37–5.40) | 0.35× (0.35–0.35) | 0.38× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.25× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.26–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.60) | 0.77× (0.76–0.77) | 5.31× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.90× (0.90–0.91) | 0.13× (0.12–0.13) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.41× (1.41–1.42) | 0.66× (0.66–0.66) | 6.40× (6.37–6.42) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 2 | 39 MB | 0.43× (0.42–0.44) | 0.37× (0.33–0.41) | 1.00× (0.97–1.03) | 0.64× (0.62–0.67) | 3.14× (3.06–3.23) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `text` (2.51 MB → 12.6 MB, 5.0×) | 4 | 56 MB | 0.49× (0.48–0.50) | 0.47× (0.41–0.54) | 0.90× (0.88–0.92) | 0.75× (0.73–0.76) | 2.31× (2.25–2.36) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 646 | 1.56 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,074 | 1.71 | 1.10× (1.05–1.14) | 84% | 1.02× (1.01–1.04) | 1.02× (1.00–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 1,308 | 2.65 | 1.75× (1.61–1.89) | 49% | 1.21× (1.17–1.25) | 1.69× (1.60–1.77) |

**Best:** mibz2 at 4 threads (1,308 MB/s); lbzip2 at 4 threads (312 MB/s); pbz2 at 4 threads (503 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.24, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.45, pbz2 4t 0.56. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 14%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 4%, pbz2 2t 5%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.41 | 0.01 | 0.08 | 0.160 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.61 | 14.4% |
| MTF + RLE2 | 0.88 | 20.7% |
| T-vector build | 0.79 | 18.5% |
| walk (the one each build uses) | 1.74 | 40.8% |
| of which: segment assembly | 0.07 | 1.7% |
| RLE1 | 0.09 | 2.2% |
| CRC | 0.15 | 3.6% |
| **total** | 4.25 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.296 | 0.717 | (in its total) | 2.41× (2.36–2.46) |  |
| walk | 0.614 | 1.589 | (in its total) | 2.60× (2.53–2.66) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.029 | (in its total) | 1.15× (1.12–1.17) |  |
| inverse BWT, total | 0.936 | 2.334 | 1.298 | 2.50× (2.44–2.56) | 1.39× (1.37–1.40) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 47m35s ± 5m34s |
| 4 threads | 24m23s ± 4m05s |
| 4 threads, the sample's overlap gap as a fixed cost | 15m43s ± 2m41s |
| the scanner alone (its serial part) | 8m44s ± 41.6s |
| the workers alone, over 4 | 15m43s ± 2m41s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.06 ns a byte of output at 4 threads (CPU, summed over them), 1.27 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,506,264 | 12,614,346 | \`520a957f38391906…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
