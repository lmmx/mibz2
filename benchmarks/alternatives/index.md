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

The latest record: [`bee4173`](<https://github.com/lmmx/mibz2/commit/bee4173decac0753a102fe557a2b526938a194eb>) (2026-11-14).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43652394139>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.520 s | 403 | 8.1 | 2.88× (2.87–2.90) | 6.58× (6.54–6.62) | 6.75× (6.71–6.79) | 7.36× (7.32–7.41) | 6.62× (6.56–6.67) | 7.21× (7.14–7.28) | 6.73× (6.69–6.78) | 2.14× (2.12–2.15) | 2.49× (2.46–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.303 s | 692 | 9.1 | 2.81× (2.72–2.89) | 6.00× (5.75–6.26) | 6.81× (6.63–6.99) | 6.78× (6.51–7.05) | 6.01× (5.80–6.23) | 12.20× (11.82–12.59) | 11.39× (11.03–11.76) | 3.62× (3.50–3.74) | 4.21× (4.07–4.35) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.255 s | 824 | 14.1 | 2.78× (2.74–2.82) | 4.34× (4.09–4.61) | 5.28× (5.19–5.38) | 4.77× (4.71–4.83) | 4.78× (4.71–4.86) | 14.81× (14.56–15.06) | 13.83× (13.63–14.03) | 4.39× (4.33–4.45) | 5.11× (5.03–5.19) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 s | 1,005 | 3.5 | 0.61× (0.60–0.62) | 3.66× (3.55–3.78) | 4.58× (4.52–4.63) | 3.65× (3.62–3.69) | 6.23× (6.13–6.33) | 3.07× (3.03–3.10) | 2.74× (2.71–2.77) | 1.02× (1.00–1.03) | 1.55× (1.54–1.57) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.020 s | 990 | 3.7 | 0.58× (0.57–0.59) | 3.06× (3.03–3.10) | 6.04× (5.97–6.12) | 3.64× (3.60–3.68) | 6.14× (6.07–6.22) | 3.02× (2.99–3.05) | 2.70× (2.67–2.73) | 1.00× (0.98–1.02) | 1.53× (1.51–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 978 | 3.6 | 0.58× (0.57–0.59) | 3.02× (2.98–3.07) | 5.95× (5.86–6.04) | 3.57× (3.51–3.62) | 6.08× (5.99–6.17) | 2.97× (2.92–3.01) | 2.65× (2.61–2.69) | 0.98× (0.97–1.00) | 1.50× (1.48–1.53) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 590 | 5.8 | 2.74× (2.68–2.80) | 3.06× (3.04–3.09) | 3.94× (3.84–4.04) | 3.85× (3.82–3.87) | 4.15× (4.11–4.18) | 2.58× (2.56–2.60) | 2.13× (2.12–2.15) | 2.04× (2.01–2.06) | 3.05× (3.03–3.07) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 899 | 7.2 | 2.27× (2.21–2.35) | 2.89× (2.72–3.06) | 4.14× (3.85–4.45) | 3.64× (3.49–3.81) | 4.80× (4.63–4.97) | 3.90× (3.78–4.02) | 3.22× (3.12–3.32) | 3.08× (2.97–3.19) | 4.60× (4.46–4.75) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 781 | 13.8 | 1.82× (1.78–1.87) | 2.04× (1.93–2.15) | 2.96× (2.84–3.08) | 2.23× (2.16–2.30) | 3.86× (3.76–3.96) | 3.42× (3.35–3.50) | 2.83× (2.76–2.90) | 2.70× (2.65–2.76) | 4.04× (3.95–4.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 53.2 | 1.38× (1.37–1.38) | 2.73× (2.70–2.76) | 2.45× (2.41–2.49) | 2.47× (2.46–2.48) | 2.87× (2.85–2.88) | 3.16× (3.15–3.17) | 3.22× (3.20–3.23) | 1.24× (1.23–1.24) | 1.41× (1.41–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.080 s | 101 | 60.7 | 1.30× (1.26–1.33) | 2.55× (2.48–2.63) | 2.40× (2.34–2.46) | 2.94× (2.86–3.02) | 2.92× (2.86–2.98) | 5.09× (4.97–5.21) | 5.18× (5.05–5.30) | 1.99× (1.95–2.04) | 2.28× (2.22–2.33) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.067 s | 120 | 81.3 | 1.35× (1.33–1.38) | 2.17× (2.12–2.22) | 2.07× (2.02–2.12) | 2.51× (2.47–2.54) | 2.85× (2.81–2.90) | 6.10× (6.01–6.19) | 6.20× (6.12–6.28) | 2.39× (2.35–2.42) | 2.73× (2.69–2.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.144 s | 67 | 49.2 | 1.49× (1.49–1.50) | 3.07× (3.04–3.10) | 2.55× (2.52–2.59) | 2.63× (2.61–2.65) | 3.05× (3.04–3.07) | 3.64× (3.62–3.65) | 3.94× (3.92–3.96) | 1.43× (1.37–1.49) | 1.69× (1.69–1.70) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.088 s | 109 | 55.9 | 1.34× (1.30–1.39) | 2.76× (2.67–2.86) | 2.50× (2.43–2.57) | 3.25× (3.13–3.37) | 2.98× (2.89–3.08) | 5.83× (5.64–6.03) | 6.32× (6.12–6.54) | 2.29× (2.16–2.43) | 2.72× (2.63–2.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.075 s | 128 | 79.4 | 1.45× (1.42–1.48) | 2.31× (2.22–2.41) | 2.21× (2.16–2.27) | 2.90× (2.85–2.95) | 2.80× (2.74–2.87) | 6.96× (6.84–7.09) | 7.55× (7.42–7.68) | 2.74× (2.58–2.89) | 3.24× (3.19–3.30) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 755 | 5.0 | 0.68× (0.66–0.70) | 3.09× (3.06–3.13) | 4.54× (4.49–4.59) | 3.16× (3.13–3.20) | 6.79× (6.72–6.86) | 2.58× (2.55–2.61) | 2.38× (2.35–2.41) | 0.99× (0.97–1.01) | 1.56× (1.54–1.57) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 709 | 5.3 | 0.57× (0.56–0.58) | 2.94× (2.89–3.00) | 5.07× (4.98–5.16) | 3.05× (2.99–3.10) | 6.47× (6.36–6.59) | 2.45× (2.41–2.50) | 2.26× (2.22–2.30) | 0.94× (0.92–0.96) | 1.48× (1.45–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 712 | 5.2 | 0.58× (0.57–0.58) | 2.94× (2.92–2.97) | 5.05× (5.01–5.10) | 3.04× (3.01–3.07) | 6.47× (6.38–6.55) | 2.44× (2.42–2.47) | 2.25× (2.23–2.27) | 0.94× (0.92–0.96) | 1.47× (1.46–1.49) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 0.066 s | 186 | 18.0 | 1.97× (1.90–2.04) | 4.13× (4.08–4.19) | 4.40× (4.34–4.46) | 4.43× (4.38–4.49) | 4.67× (4.60–4.74) | 5.21× (5.14–5.29) | 4.87× (4.80–4.94) | 1.50× (1.48–1.52) | 1.83× (1.80–1.86) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 0.046 s | 267 | 22.1 | 1.51× (1.39–1.64) | 3.20× (2.98–3.42) | 3.64× (3.38–3.93) | 4.22× (4.03–4.43) | 3.99× (3.73–4.26) | 7.22× (6.74–7.73) | 6.74× (6.31–7.21) | 2.08× (1.94–2.22) | 2.54× (2.37–2.71) | yes |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 0.044 s | 282 | 34.4 | 1.42× (1.37–1.47) | 2.24× (2.11–2.38) | 2.85× (2.72–3.00) | 3.21× (3.12–3.30) | 3.39× (3.31–3.47) | 7.92× (7.75–8.09) | 7.40× (7.24–7.56) | 2.28× (2.23–2.32) | 2.78× (2.72–2.84) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.88× (2.86–2.89) | 6.55× (6.51–6.59) | 6.72× (6.68–6.77) | 7.34× (7.30–7.39) | 6.60× (6.54–6.65) | 7.19× (7.11–7.26) | 6.71× (6.67–6.75) | 2.13× (2.12–2.15) | 2.48× (2.45–2.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.69× (2.66–2.72) | 6.04× (5.99–6.09) | 6.36× (6.30–6.43) | 6.96× (6.83–7.09) | 5.97× (5.89–6.05) | 6.42× (6.33–6.50) | 5.99× (5.92–6.07) | 1.91× (1.88–1.93) | 2.21× (2.19–2.23) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.74× (2.70–2.79) | 4.59× (4.52–4.66) | 4.94× (4.87–5.01) | 5.18× (5.12–5.25) | 4.57× (4.50–4.64) | 4.14× (4.06–4.21) | 3.86× (3.81–3.91) | 1.23× (1.22–1.24) | 1.43× (1.40–1.45) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.65) | 3.37× (3.33–3.41) | 4.15× (4.11–4.20) | 3.42× (3.38–3.46) | 5.77× (5.68–5.87) | 2.88× (2.85–2.92) | 2.59× (2.55–2.62) | 1.02× (1.00–1.03) | 1.06× (1.04–1.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.63× (0.62–0.63) | 3.14× (3.10–3.18) | 5.42× (5.37–5.48) | 3.37× (3.34–3.40) | 5.60× (5.54–5.66) | 2.79× (2.76–2.82) | 2.50× (2.48–2.53) | 0.98× (0.97–1.00) | 1.03× (1.01–1.04) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.62× (0.61–0.63) | 3.44× (3.36–3.52) | 5.42× (5.33–5.51) | 3.37× (3.31–3.42) | 5.62× (5.53–5.70) | 2.79× (2.75–2.84) | 2.50× (2.46–2.54) | 0.98× (0.97–1.00) | 1.03× (1.01–1.05) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.64× (2.58–2.70) | 2.82× (2.80–2.84) | 3.56× (3.46–3.66) | 3.69× (3.67–3.71) | 3.97× (3.95–4.00) | 2.50× (2.48–2.51) | 2.07× (2.06–2.09) | 1.98× (1.96–2.00) | 2.71× (2.69–2.74) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.26× (2.20–2.32) | 2.50× (2.45–2.56) | 3.09× (2.95–3.24) | 3.33× (3.19–3.48) | 3.34× (3.26–3.42) | 2.01× (1.97–2.06) | 1.67× (1.63–1.71) | 1.60× (1.55–1.64) | 2.19× (2.13–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.04× (1.96–2.13) | 1.79× (1.70–1.88) | 2.24× (2.15–2.34) | 2.17× (2.07–2.27) | 2.12× (2.03–2.22) | 1.05× (1.01–1.10) | 0.87× (0.84–0.91) | 0.83× (0.80–0.87) | 1.14× (1.09–1.20) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.66× (2.65–2.68) | 2.40× (2.37–2.43) | 2.45× (2.44–2.46) | 2.84× (2.82–2.85) | 3.13× (3.12–3.13) | 3.18× (3.17–3.19) | 1.23× (1.23–1.24) | 1.38× (1.37–1.39) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.25–1.28) | 2.46× (2.43–2.48) | 2.17× (2.14–2.19) | 2.77× (2.73–2.81) | 2.54× (2.51–2.57) | 2.73× (2.70–2.77) | 2.78× (2.74–2.82) | 1.08× (1.07–1.09) | 1.21× (1.19–1.22) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.48× (1.42–1.55) | 2.33× (2.30–2.36) | 1.88× (1.86–1.91) | 2.48× (2.46–2.50) | 2.43× (2.41–2.46) | 2.04× (2.02–2.06) | 2.08× (2.06–2.09) | 0.81× (0.80–0.81) | 0.90× (0.89–0.91) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.48–1.49) | 3.00× (2.97–3.03) | 2.51× (2.48–2.54) | 2.61× (2.59–2.63) | 3.03× (3.01–3.05) | 3.60× (3.58–3.62) | 3.90× (3.88–3.92) | 1.42× (1.36–1.49) | 1.66× (1.65–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.36–1.40) | 2.78× (2.75–2.81) | 2.35× (2.30–2.40) | 3.13× (3.09–3.18) | 2.74× (2.71–2.76) | 3.18× (3.16–3.20) | 3.44× (3.42–3.47) | 1.26× (1.20–1.31) | 1.46× (1.45–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.57–1.66) | 2.51× (2.45–2.57) | 1.94× (1.91–1.97) | 2.70× (2.63–2.77) | 2.52× (2.47–2.56) | 2.25× (2.21–2.29) | 2.44× (2.39–2.48) | 0.89× (0.85–0.93) | 1.04× (1.02–1.05) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.70–0.74) | 2.74× (2.72–2.77) | 3.59× (3.54–3.64) | 2.88× (2.84–2.92) | 6.04× (5.97–6.11) | 2.38× (2.35–2.40) | 2.20× (2.17–2.23) | 0.99× (0.97–1.01) | 0.98× (0.96–1.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.63× (0.62–0.64) | 2.55× (2.50–2.59) | 4.30× (4.23–4.36) | 2.79× (2.74–2.84) | 5.68× (5.59–5.77) | 2.22× (2.18–2.25) | 2.05× (2.02–2.09) | 0.92× (0.91–0.94) | 0.92× (0.89–0.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.62× (0.62–0.63) | 2.82× (2.79–2.85) | 4.39× (4.32–4.46) | 2.87× (2.85–2.89) | 5.84× (5.76–5.91) | 2.27× (2.25–2.30) | 2.10× (2.08–2.13) | 0.95× (0.93–0.97) | 0.94× (0.92–0.97) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 1.94× (1.88–2.01) | 3.92× (3.87–3.97) | 4.23× (4.18–4.28) | 4.33× (4.28–4.39) | 4.56× (4.50–4.63) | 5.09× (5.01–5.16) | 4.75× (4.68–4.82) | 1.48× (1.46–1.51) | 1.77× (1.74–1.80) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 1.65× (1.61–1.70) | 3.30× (3.23–3.38) | 3.63× (3.54–3.73) | 3.98× (3.88–4.08) | 3.76× (3.67–3.86) | 4.09× (3.98–4.21) | 3.83× (3.72–3.93) | 1.19× (1.16–1.23) | 1.42× (1.38–1.47) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 1.73× (1.64–1.83) | 2.64× (2.59–2.68) | 2.90× (2.83–2.97) | 3.11× (3.05–3.18) | 3.05× (2.99–3.10) | 2.67× (2.63–2.72) | 2.50× (2.45–2.55) | 0.78× (0.76–0.80) | 0.93× (0.91–0.95) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 33 MB | 0.26× (0.26–0.26) | 0.26× (0.25–0.27) | 0.84× (0.84–0.85) | 0.67× (0.67–0.68) | 9.55× (9.53–9.56) | 0.23× (0.22–0.23) | 0.25× (0.24–0.25) | 0.37× (0.37–0.37) | 0.19× (0.18–0.19) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 78 MB | 0.25× (0.24–0.25) | 0.34× (0.31–0.36) | 0.51× (0.49–0.53) | 0.43× (0.41–0.44) | 4.05× (3.92–4.19) | 0.10× (0.09–0.10) | 0.10× (0.10–0.11) | 0.16× (0.15–0.16) | 0.08× (0.08–0.08) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 118 MB | 0.25× (0.23–0.28) | 0.39× (0.34–0.45) | 0.45× (0.41–0.50) | 0.39× (0.36–0.43) | 2.63× (2.37–2.91) | 0.06× (0.05–0.07) | 0.07× (0.06–0.07) | 0.10× (0.09–0.11) | 0.05× (0.05–0.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.17–0.18) | 0.82× (0.81–0.82) | 0.81× (0.80–0.81) | 4.29× (4.28–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.83× (0.83–0.83) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.83× (0.83–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.70× (1.70–1.71) | 0.65× (0.65–0.66) | 8.03× (8.01–8.05) | 0.65× (0.65–0.65) | 0.69× (0.68–0.70) | 0.75× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.65–0.68) | 0.75× (0.69–0.82) | 1.61× (1.58–1.65) | 0.80× (0.78–0.82) | 5.83× (5.70–5.97) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.56) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.63× (0.60–0.66) | 0.71× (0.66–0.77) | 1.13× (1.08–1.19) | 0.69× (0.66–0.72) | 3.33× (3.21–3.47) | 0.26× (0.25–0.27) | 0.28× (0.26–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.73) | 5.22× (5.21–5.22) | 0.33× (0.32–0.33) | 0.35× (0.35–0.35) | 0.53× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.49–0.52) | 0.40× (0.37–0.43) | 0.90× (0.89–0.92) | 0.70× (0.67–0.73) | 3.12× (3.07–3.17) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.31× (0.31–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.85× (0.83–0.88) | 0.79× (0.76–0.83) | 2.49× (2.42–2.56) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.31–0.32) | 1.19× (1.18–1.19) | 0.83× (0.83–0.83) | 5.23× (5.22–5.23) | 0.32× (0.32–0.32) | 0.35× (0.34–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.50) | 0.37× (0.35–0.40) | 0.87× (0.84–0.90) | 0.72× (0.68–0.75) | 2.96× (2.85–3.07) | 0.18× (0.18–0.19) | 0.20× (0.19–0.21) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.47× (0.45–0.50) | 0.80× (0.79–0.81) | 0.82× (0.79–0.85) | 2.31× (2.28–2.33) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.21× (0.20–0.21) | 1.08× (1.08–1.08) | 0.76× (0.76–0.77) | 5.39× (5.36–5.41) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.93) | 0.13× (0.13–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.76× (0.76–0.77) | 5.30× (5.28–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.61× (1.61–1.62) | 0.77× (0.76–0.77) | 5.34× (5.33–5.35) | 0.35× (0.34–0.35) | 0.37× (0.36–0.38) | 0.91× (0.91–0.91) | 0.13× (0.13–0.13) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.42–1.43) | 0.66× (0.65–0.66) | 6.41× (6.40–6.43) | 0.39× (0.39–0.40) | 0.43× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.42× (0.38–0.47) | 1.16× (1.14–1.19) | 0.73× (0.69–0.77) | 3.61× (3.53–3.69) | 0.22× (0.22–0.23) | 0.24× (0.23–0.25) | 0.36× (0.35–0.37) | 0.17× (0.17–0.18) |
| `text` (2.44 MB → 12.3 MB, 5.0×) | 4 | 54 MB | 0.52× (0.51–0.53) | 0.52× (0.49–0.56) | 0.95× (0.93–0.97) | 0.77× (0.74–0.80) | 2.43× (2.38–2.48) | 0.14× (0.14–0.14) | 0.15× (0.15–0.16) | 0.23× (0.22–0.23) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 403 | 2.49 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 692 | 2.79 | 1.12× (1.11–1.13) | 85% | 1.03× (1.03–1.04) | 1.05× (1.05–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 824 | 4.33 | 1.74× (1.71–1.76) | 51% | 1.22× (1.21–1.23) | 1.66× (1.64–1.68) |

**Best:** mibz2 at 4 threads (824 MB/s); lbzip2 at 4 threads (196 MB/s); pbz2 at 4 threads (298 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.74, mibz2 4t 1.13, lbzip2 1t 0.09, lbzip2 2t 0.36, lbzip2 4t 0.50, pbz2 1t 0.00, pbz2 2t 0.33, pbz2 4t 0.43. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 9%, lbzip2 2t 5%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.71 | 0.02 | 0.08 | 0.255 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3244 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.11 | 12.5% |
| MTF + RLE2 | 1.59 | 17.9% |
| T-vector build | 1.49 | 16.8% |
| walk (the one each build uses) | 4.16 | 46.8% |
| of which: segment assembly | 0.14 | 1.6% |
| RLE1 | 0.28 | 3.1% |
| CRC | 0.26 | 2.9% |
| **total** | 8.88 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.486 | 1.301 | (in its total) | 2.68× (2.67–2.68) |  |
| walk | 1.213 | 2.628 | (in its total) | 2.16× (2.15–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.044 | 0.054 | (in its total) | 1.24× (1.23–1.25) |  |
| inverse BWT, total | 1.743 | 3.985 | 3.266 | 2.28× (2.27–2.29) | 1.87× (1.86–1.88) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m44s |
| 4 threads | 36m57s ± 5m12s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m52s ± 3m40s |
| the scanner alone (its serial part) | 13m41s ± 32.2s |
| the workers alone, over 4 | 25m52s ± 3m40s |

At 4 threads the workers bound it. Per byte: the scanner 7.9 ns a byte of bz2, the workers 3.39 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,443,515 | 12,305,874 | \`da24cf0eff3da816…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
